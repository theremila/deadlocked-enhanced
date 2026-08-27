use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::{LazyLock, Mutex},
};

use crate::{
    config::BASE_PATH,
    cs2::{CS2, bvh::read_bvh},
    parser::bvh::Bvh,
};

pub mod bvh;
mod gltf;

#[derive(Default)]
struct MaterialBuildState {
    loading: Option<String>,
    ready: Option<(String, Bvh)>,
}

static MATERIAL_BUILD: LazyLock<Mutex<MaterialBuildState>> =
    LazyLock::new(|| Mutex::new(MaterialBuildState::default()));

pub fn read_map(cs2: &CS2, map_name: &str) -> Option<Bvh> {
    let map_name = map_name.trim_end_matches(".vpk");
    if map_name.is_empty() {
        return runtime_bvh(cs2);
    }

    let cache_dir = BASE_PATH.join("material_maps");
    let cache_path = cache_dir.join(format!("{map_name}.bvh"));
    if let Some(bvh) = Bvh::load(&cache_path) {
        utils::info!("loaded material BVH cache for {map_name}");
        return Some(bvh);
    }

    let game_dir = find_game_dir(cs2);
    request_material_bvh(map_name.to_owned(), cache_dir, game_dir);
    runtime_bvh(cs2)
}

pub fn take_material_bvh(map_name: &str) -> Option<Bvh> {
    let mut state = MATERIAL_BUILD.lock().ok()?;
    if state.ready.as_ref().is_some_and(|(map, _)| map == map_name) {
        return state.ready.take().map(|(_, bvh)| bvh);
    }
    None
}

fn request_material_bvh(map_name: String, cache_dir: PathBuf, game_dir: Option<PathBuf>) {
    let Ok(mut state) = MATERIAL_BUILD.lock() else {
        return;
    };
    if state.loading.as_deref() == Some(&map_name)
        || state
            .ready
            .as_ref()
            .is_some_and(|(map, _)| map == &map_name)
    {
        return;
    }
    state.loading = Some(map_name.clone());
    std::thread::spawn(move || {
        let bvh =
            game_dir.and_then(|game_dir| build_material_bvh(&map_name, &cache_dir, &game_dir));
        if let Some(bvh) = &bvh
            && std::fs::create_dir_all(&cache_dir).is_ok()
            && bvh
                .save(&cache_dir.join(format!("{map_name}.bvh")))
                .is_none()
        {
            utils::warn!("failed to cache material BVH for {map_name}");
        }
        if bvh.is_none() {
            utils::warn!("material map unavailable for {map_name}; keeping runtime geometry");
        }
        if let Ok(mut state) = MATERIAL_BUILD.lock() {
            state.loading = None;
            state.ready = bvh.map(|bvh| (map_name, bvh));
        }
    });
}

fn runtime_bvh(cs2: &CS2) -> Option<Bvh> {
    let triangles = read_bvh(cs2)?;
    let mut bvh = Bvh::new();
    bvh.set(triangles);
    bvh.build();
    Some(bvh)
}

fn build_material_bvh(map_name: &str, cache_dir: &Path, game_dir: &Path) -> Option<Bvh> {
    let map_vpk = game_dir
        .join("game/csgo/maps")
        .join(format!("{map_name}.vpk"));
    if !map_vpk.exists() {
        utils::warn!("map VPK not found: {}", map_vpk.display());
        return None;
    }

    let viewer = source2viewer_binary()?;
    let output_dir = cache_dir.join(format!("{map_name}.source2viewer"));
    if std::fs::create_dir_all(&output_dir).is_err() {
        return None;
    }

    utils::info!("extracting material geometry for {map_name}");
    let result = Command::new(viewer)
        .args([
            "-i",
            map_vpk.to_string_lossy().as_ref(),
            "-d",
            "-o",
            output_dir.to_string_lossy().as_ref(),
            "-f",
            &format!("maps/{map_name}/world_physics.vmdl_c"),
            "--gltf_export_format",
            "gltf",
            "--gltf_export_extras",
        ])
        .output()
        .ok()?;
    if !result.status.success() {
        utils::warn!(
            "Source2Viewer failed for {map_name}: {}",
            String::from_utf8_lossy(&result.stderr).trim()
        );
        return None;
    }

    let gltf_path = output_dir
        .join("maps")
        .join(map_name)
        .join("world_physics_physics.gltf");
    let bvh = gltf::load_material_bvh(&gltf_path);
    if bvh.is_some() {
        utils::info!("built material-aware BVH for {map_name}");
        let _ = std::fs::remove_dir_all(&output_dir);
    }
    bvh
}

fn source2viewer_binary() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("SOURCE2VIEWER_PATH").map(PathBuf::from)
        && path.exists()
    {
        return Some(path);
    }
    if let Some(executable) = std::env::current_exe().ok()
        && let Some(path) = executable.ancestors().find_map(|directory| {
            let candidate = directory.join("resources/source2viewer/Source2Viewer-CLI");
            candidate.is_file().then_some(candidate)
        })
    {
        return Some(path);
    }
    Command::new("Source2Viewer-CLI")
        .arg("--version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|_| PathBuf::from("Source2Viewer-CLI"))
}

fn find_game_dir(cs2: &CS2) -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("CS2_GAME_DIR").map(PathBuf::from)
        && path.join("game/csgo").exists()
    {
        return Some(path);
    }

    let executable = cs2.executable_path()?;
    executable
        .ancestors()
        .find_map(|path| path.join("game/csgo").exists().then(|| path.to_path_buf()))
}
