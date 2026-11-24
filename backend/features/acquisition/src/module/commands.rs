use sweepscope_kernel::CommandDescriptor;

/// Frontend-callable commands of this feature, consumed by the shell to generate `commands.ts`.
pub const COMMANDS: &[CommandDescriptor] = &[
    CommandDescriptor {
        name: "set_running",
        args: "{ running: boolean }",
        result: "null",
        types: &[],
    },
    CommandDescriptor {
        name: "force_trigger",
        args: "{}",
        result: "null",
        types: &[],
    },
    CommandDescriptor {
        name: "set_channel",
        args: "{ channel: number; patch: ChannelPatch }",
        result: "null",
        types: &["ChannelPatch"],
    },
    CommandDescriptor {
        name: "step_setting",
        args: "{ target: SettingTarget; direction: -1 | 1 }",
        result: "null",
        types: &["SettingTarget"],
    },
    CommandDescriptor {
        name: "reset_setting",
        args: "{ target: SettingTarget }",
        result: "null",
        types: &["SettingTarget"],
    },
    CommandDescriptor {
        name: "set_trigger_position",
        args: "{ fraction: number }",
        result: "null",
        types: &[],
    },
    CommandDescriptor {
        name: "set_trigger_level",
        args: "{ divisions: number }",
        result: "null",
        types: &[],
    },
    CommandDescriptor {
        name: "set_trigger",
        args: "{ patch: TriggerPatch }",
        result: "null",
        types: &["TriggerPatch"],
    },
    CommandDescriptor {
        name: "save_setup",
        args: "{}",
        result: "SavedFile | null",
        types: &["SavedFile"],
    },
    CommandDescriptor {
        name: "load_setup",
        args: "{}",
        result: "SetupLoaded | null",
        types: &["SetupLoaded"],
    },
    CommandDescriptor {
        name: "arm_single",
        args: "{}",
        result: "null",
        types: &[],
    },
    CommandDescriptor {
        name: "start_autoset",
        args: "{}",
        result: "null",
        types: &[],
    },
    CommandDescriptor {
        name: "set_math",
        args: "{ patch: MathPatch }",
        result: "null",
        types: &["MathPatch"],
    },
];
