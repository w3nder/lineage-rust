use killer_assets::cache::SourceStamp;
use killer_assets::client::Client;
use std::time::UNIX_EPOCH;

pub fn stamp(client: &mut Client, raw: &[u8]) -> SourceStamp {
    with_revision(client, raw, env!("KILLER_GIT_HASH"))
}

fn with_revision(client: &mut Client, raw: &[u8], revision: &str) -> SourceStamp {
    let mut stamp = SourceStamp::of(raw);
    stamp.include(b"scene-cache-context-v1");
    stamp.include(revision.as_bytes());
    for name in ["option.ini", "l2.ini", "env.int"] {
        stamp.include(name.as_bytes());
        match client.read_system_text(name) {
            Ok(text) => { stamp.include(b"present"); stamp.include(text.as_bytes()); }
            Err(_) => stamp.include(b"absent"),
        }
    }
    let mut paths: Vec<_> = client.catalog().source_paths().collect();
    paths.sort_unstable();
    for path in paths {
        stamp.include(path.to_string_lossy().as_bytes());
        match std::fs::metadata(path) {
            Ok(metadata) => {
                stamp.include(b"present");
                stamp.include(&metadata.len().to_le_bytes());
                let modified = metadata.modified().ok().and_then(|time| time.duration_since(UNIX_EPOCH).ok());
                stamp.include(&modified.map(|time| time.as_nanos()).unwrap_or_default().to_le_bytes());
            }
            Err(_) => stamp.include(b"absent"),
        }
    }
    stamp
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_assets::cache::{CachedScene, SceneCache};
    use killer_assets::scene::Scene;

    #[test]
    fn cache_rejects_other_builds_options_and_changed_resource_packages() {
        let root = std::env::temp_dir().join(format!("killer-scene-context-{}", std::process::id()));
        std::fs::create_dir_all(root.join("system")).unwrap();
        std::fs::create_dir_all(root.join("Textures")).unwrap();
        let package = root.join("Textures/alpha.utx");
        std::fs::write(&package, b"initial").unwrap();
        let mut client = Client::open(&root).unwrap();
        let map = root.join("map.unr");
        let cache = SceneCache::at(root.join("cache"));
        let stamp = with_revision(&mut client, b"map", "build-a");
        let scene = CachedScene { scene: Scene::default(), textures: Vec::new(), alpha_maps: Vec::new() };
        cache.write(&map, &stamp, &scene).unwrap();
        assert_eq!(cache.read(&map, &with_revision(&mut client, b"map", "build-a")), Some(scene));
        assert!(cache.read(&map, &with_revision(&mut client, b"map", "build-b")).is_none());
        assert!(cache.read(&map, &with_revision(&mut client, b"other-map", "build-a")).is_none());
        std::fs::write(root.join("system/option.ini"), b"[Video]\nIsKeepMinFrameRate=True\n").unwrap();
        assert!(cache.read(&map, &with_revision(&mut client, b"map", "build-a")).is_none());
        std::fs::remove_file(root.join("system/option.ini")).unwrap();
        assert_eq!(with_revision(&mut client, b"map", "build-a"), stamp);
        std::fs::write(&package, b"updated-resource").unwrap();
        assert!(cache.read(&map, &with_revision(&mut client, b"map", "build-a")).is_none());
        std::fs::remove_dir_all(root).unwrap();
    }
}
