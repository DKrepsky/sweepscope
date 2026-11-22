/// Describes one frontend-callable command so the shell can generate the TypeScript manifest from one list.
/// `args` and `result` are TypeScript type expressions; `types` names the generated types they use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandDescriptor {
    pub name: &'static str,
    pub args: &'static str,
    pub result: &'static str,
    pub types: &'static [&'static str],
}
