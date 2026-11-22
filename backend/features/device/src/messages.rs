use sweepscope_kernel::contracts::ConnectionSnapshot;
use sweepscope_kernel::Request;

/// Plugs the simulator in or out as a device source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UseSimulator {
    pub enabled: bool,
}
impl Request for UseSimulator {
    type Response = ();
}

/// Tries to open the attached device again after an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryConnection;
impl Request for RetryConnection {
    type Response = ();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GetConnection;
impl Request for GetConnection {
    type Response = ConnectionSnapshot;
}
