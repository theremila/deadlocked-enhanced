pub mod cs2 {
    use crate::cs2::entity::weapon::Weapon;

    pub const PROCESS_NAME: &str = "cs2";
    pub const CLIENT_LIB: &str = "libclient.so";
    pub const ENGINE_LIB: &str = "libengine2.so";
    pub const TIER0_LIB: &str = "libtier0.so";
    pub const INPUT_LIB: &str = "libinputsystem.so";
    pub const SDL_LIB: &str = "libSDL3.so.0";
    pub const SCHEMA_LIB: &str = "libschemasystem.so";

    pub const LIBS: [&str; 6] = [
        CLIENT_LIB, ENGINE_LIB, TIER0_LIB, INPUT_LIB, SDL_LIB, SCHEMA_LIB,
    ];

    pub const TEAM_T: u8 = 2;
    pub const TEAM_CT: u8 = 3;

    pub const WEAPON_UNKNOWN: &str = "unknown";
    pub const DEFAULT_FOV: u32 = 90;

    pub const SOUND_ESP_FOOTSTEP_DIAMETER_DEFAULT: f32 = 2000.0;
    pub const SOUND_ESP_GUNSHOT_DIAMETER_DEFAULT: f32 = 3000.0;
    pub const SOUND_ESP_WEAPON_DIAMETER_DEFAULT: f32 = 1000.0;

    pub const GRENADES: &[Weapon] = &[
        Weapon::Decoy,
        Weapon::Flashbang,
        Weapon::HeGrenade,
        Weapon::Incendiary,
        Weapon::Molotov,
        Weapon::Smoke,
    ];

    pub mod class {
        pub const PLAYER_CONTROLLER: &str = "19CCSPlayerController";

        pub const PLANTED_C4: &str = "11C_PlantedC4";
        pub const INFERNO: &str = "9C_Inferno";
        pub const SMOKE: &str = "24C_SmokeGrenadeProjectile";
        pub const MOLOTOV: &str = "19C_MolotovProjectile";
        pub const FLASHBANG: &str = "21C_FlashbangProjectile";
        pub const HE_GRENADE: &str = "21C_HEGrenadeProjectile";
        pub const DECOY: &str = "17C_DecoyProjectile";

        pub const CHICKEN: &str = "9C_Chicken";
    }
}

pub mod elf {
    pub const PROGRAM_HEADER_OFFSET: usize = 0x20;
    pub const PROGRAM_HEADER_ENTRY_SIZE: usize = 0x36;
    pub const PROGRAM_HEADER_NUM_ENTRIES: usize = 0x38;

    pub const SECTION_HEADER_OFFSET: usize = 0x28;
    pub const SECTION_HEADER_ENTRY_SIZE: usize = 0x3A;
    pub const SECTION_HEADER_NUM_ENTRIES: usize = 0x3C;

    pub const DYNAMIC_SECTION_PHT_TYPE: usize = 0x02;
}

pub const GRENADE_FILE_NAME: &str = "grenades.json";

pub mod timing {
    use std::time::Duration;

    pub const ENTITY_CACHE_INTERVAL: Duration = Duration::from_millis(200);
    pub const MAINTENANCE_INTERVAL: Duration = Duration::from_millis(8);
    pub const RUNTIME_PUBLISH_INTERVAL: Duration = Duration::from_millis(2);
    pub const FRAME_TIME_REPORT_INTERVAL: Duration = Duration::from_millis(250);
    pub const SLOW_LOOP_WARNING: Duration = Duration::from_millis(2);
    pub const INVALID_PROCESS_RETRY_INTERVAL: Duration = Duration::from_secs(1);
    pub const ESP_FRAME_INTERVAL: Duration = Duration::from_millis(8);

    pub const BHOP_FULL_RETRY_INTERVAL: Duration = Duration::from_millis(2);
    pub const BHOP_LEGIT_RETRY_INTERVAL: Duration = Duration::from_millis(6);

    // Aim/RCS tuning historically assumed one update after a 2 ms loop sleep.
    // Keep that response curve as the time-normalization reference.
    pub const AIM_REFERENCE_INTERVAL: Duration = Duration::from_millis(2);
    pub const AIM_MAX_TIMESTEP: Duration = Duration::from_millis(8);
    pub const RCS_UPDATE_INTERVAL: Duration = Duration::from_millis(2);
}
