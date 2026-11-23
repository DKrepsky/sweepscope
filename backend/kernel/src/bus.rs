use std::any::{type_name, Any, TypeId};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, Weak};
use std::time::{Duration, Instant};

use crate::KernelError;

const SUBSCRIBER_CAPACITY: usize = 256;

pub trait Event: Clone + Send + 'static {}

pub trait Request: Send + 'static {
    type Response: Send + 'static;
}

type Handler<R> = dyn Fn(R) -> Result<<R as Request>::Response, KernelError> + Send + Sync;

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

struct Queue<E> {
    items: Mutex<VecDeque<E>>,
    arrived: Condvar,
    lagged: AtomicU64,
    capacity: usize,
}

impl<E> Queue<E> {
    fn push(&self, event: E) {
        let mut items = lock(&self.items);
        if items.len() == self.capacity {
            items.pop_front();
            self.lagged.fetch_add(1, Ordering::Relaxed);
        }
        items.push_back(event);
        self.arrived.notify_one();
    }

    fn with_capacity(capacity: usize) -> Arc<Self> {
        Arc::new(Self {
            items: Mutex::new(VecDeque::new()),
            arrived: Condvar::new(),
            lagged: AtomicU64::new(0),
            capacity,
        })
    }
}

/// Receiving end of an event subscription; dropping it unsubscribes.
pub struct Subscription<E> {
    queue: Arc<Queue<E>>,
}

impl<E> Subscription<E> {
    pub fn try_next(&self) -> Option<E> {
        lock(&self.queue.items).pop_front()
    }

    /// Waits up to `wait` for the next event; wakes as soon as one is published.
    pub fn recv_timeout(&self, wait: Duration) -> Option<E> {
        let deadline = Instant::now() + wait;
        let mut items = lock(&self.queue.items);
        loop {
            if let Some(event) = items.pop_front() {
                return Some(event);
            }
            let remaining = deadline.checked_duration_since(Instant::now())?;
            items = self
                .queue
                .arrived
                .wait_timeout(items, remaining)
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .0;
        }
    }

    pub fn drain(&self) -> Vec<E> {
        lock(&self.queue.items).drain(..).collect()
    }

    /// Number of events dropped because this subscriber did not keep up.
    pub fn lagged(&self) -> u64 {
        self.queue.lagged.load(Ordering::Relaxed)
    }
}

#[derive(Default)]
struct Registry {
    subscribers: HashMap<TypeId, Box<dyn Any + Send>>,
    handlers: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

/// In-process message bus. Cheap to clone; all clones share the same registry.
#[derive(Clone, Default)]
pub struct Bus {
    registry: Arc<Mutex<Registry>>,
}

impl Bus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn publish<E: Event>(&self, event: E) {
        let mut registry = lock(&self.registry);
        let Some(entry) = registry.subscribers.get_mut(&TypeId::of::<E>()) else {
            return;
        };
        let Some(queues) = entry.downcast_mut::<Vec<Weak<Queue<E>>>>() else {
            return;
        };
        queues.retain(|queue| queue.strong_count() > 0);
        for queue in queues.iter().filter_map(Weak::upgrade) {
            queue.push(event.clone());
        }
    }

    pub fn subscribe<E: Event>(&self) -> Subscription<E> {
        self.subscribe_with_capacity(SUBSCRIBER_CAPACITY)
    }

    /// Keeps only the newest event; replaced ones count in `lagged()` (latest-wins streams such as frames).
    pub fn subscribe_latest<E: Event>(&self) -> Subscription<E> {
        self.subscribe_with_capacity(1)
    }

    fn subscribe_with_capacity<E: Event>(&self, capacity: usize) -> Subscription<E> {
        let queue = Queue::with_capacity(capacity);
        let mut registry = lock(&self.registry);
        let entry = registry
            .subscribers
            .entry(TypeId::of::<E>())
            .or_insert_with(|| Box::new(Vec::<Weak<Queue<E>>>::new()));
        if let Some(queues) = entry.downcast_mut::<Vec<Weak<Queue<E>>>>() {
            queues.push(Arc::downgrade(&queue));
        }
        Subscription { queue }
    }

    /// Registers the single handler of a request type; a second registration is refused and changes nothing.
    pub fn handle<R: Request>(
        &self,
        handler: impl Fn(R) -> Result<R::Response, KernelError> + Send + Sync + 'static,
    ) -> Result<(), KernelError> {
        let mut registry = lock(&self.registry);
        let key = TypeId::of::<R>();
        if registry.handlers.contains_key(&key) {
            return Err(KernelError::duplicate_handler(type_name::<R>()));
        }
        let handler: Arc<Handler<R>> = Arc::new(handler);
        registry.handlers.insert(key, Box::new(handler));
        Ok(())
    }

    pub fn request<R: Request>(&self, request: R) -> Result<R::Response, KernelError> {
        let handler = self.handler_for::<R>()?;
        handler(request)
    }

    fn handler_for<R: Request>(&self) -> Result<Arc<Handler<R>>, KernelError> {
        lock(&self.registry)
            .handlers
            .get(&TypeId::of::<R>())
            .and_then(|handler| handler.downcast_ref::<Arc<Handler<R>>>())
            .cloned()
            .ok_or_else(|| KernelError::no_handler(type_name::<R>()))
    }
}
