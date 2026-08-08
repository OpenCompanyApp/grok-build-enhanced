use super::*;

fn counted(size: DirSize) -> u64 {
    size.measure.bytes().expect("measured on its own volume")
}

#[cfg(unix)]
#[test]
fn sizing_is_physical_not_logical() {
    let tmp = tempfile::TempDir::new().unwrap();
    const LOGICAL: u64 = 1 << 20;

    let sparse = tmp.path().join("sparse.bin");
    std::fs::File::create(&sparse)
        .unwrap()
        .set_len(LOGICAL)
        .unwrap();
    let sparse_meta = std::fs::symlink_metadata(&sparse).unwrap();
    assert_eq!(sparse_meta.len(), LOGICAL);
    assert!(physical_file_size(&sparse_meta) < LOGICAL);

    let linked = tmp.path().join("linked");
    std::fs::create_dir(&linked).unwrap();
    let original = linked.join("a.bin");
    std::fs::write(&original, vec![b'x'; 8192]).unwrap();
    std::fs::hard_link(&original, linked.join("b.bin")).unwrap();
    let one_link = physical_file_size(&std::fs::symlink_metadata(&original).unwrap());
    assert_eq!(
        counted(physical_dir_size(&linked, Volume::of(&linked))),
        one_link * 2,
        "each hard link counts at full size"
    );
}

#[test]
fn physical_dir_size_sums_files_without_following_symlinks() {
    let tmp = tempfile::TempDir::new().unwrap();
    let target = tmp.path().join("target");
    std::fs::create_dir_all(target.join("sub")).unwrap();
    std::fs::write(target.join("a.bin"), vec![b'x'; 8192]).unwrap();
    std::fs::write(target.join("sub/b.bin"), vec![b'y'; 4096]).unwrap();

    let full = counted(physical_dir_size(&target, Volume::of(&target)));
    let expected: u64 = [target.join("a.bin"), target.join("sub/b.bin")]
        .iter()
        .map(|path| physical_file_size(&std::fs::symlink_metadata(path).unwrap()))
        .sum();
    assert_eq!(full, expected);

    #[cfg(unix)]
    {
        let linked = tmp.path().join("linked");
        std::fs::create_dir_all(&linked).unwrap();
        std::os::unix::fs::symlink(&target, linked.join("escape")).unwrap();
        let link_only = counted(physical_dir_size(&linked, Volume::of(&linked)));
        let link_meta = std::fs::symlink_metadata(linked.join("escape")).unwrap();
        assert_eq!(link_only, physical_file_size(&link_meta));
    }
}

#[test]
fn missing_root_counts_one_unreadable_dir() {
    let missing = Path::new("/nonexistent-grok-du-root");
    let size = physical_dir_size(missing, Volume::of(missing));
    assert_eq!(size.measure.bytes(), Some(0));
    assert_eq!(size.issues.unreadable_dirs, 1);
    assert_eq!(size.issues.skipped(), 1);
}

#[test]
fn a_volume_holds_only_its_own_device() {
    let tmp = tempfile::TempDir::new().unwrap();
    let path = tmp.path();
    assert!(Volume(None).holds(path));
    assert!(Volume::of(path).holds(Path::new("/nonexistent-grok-du-path")));
    assert!(Volume::of(path).holds(path));
    #[cfg(unix)]
    assert!(!Volume::of(path).other_device_for_test().holds(path));
}

#[cfg(unix)]
#[test]
fn a_root_off_the_anchor_is_measured_by_nobody() {
    let tmp = tempfile::TempDir::new().unwrap();
    let root = tmp.path().join("worktrees");
    let worktree = root.join("xai/wt-a");
    std::fs::create_dir_all(&worktree).unwrap();
    std::fs::write(worktree.join("payload.bin"), vec![b'x'; 65536]).unwrap();
    let elsewhere = Volume::of(tmp.path()).other_device_for_test();

    let home = physical_buckets(&root, Volume::of(&root));
    assert!(home.total.bytes().is_some_and(|bytes| bytes >= 65536));
    assert!(
        home.buckets[&worktree]
            .bytes()
            .is_some_and(|bytes| bytes >= 65536)
    );

    let foreign = physical_buckets(&root, elsewhere);
    assert_eq!(foreign.total, Measure::Elsewhere);
    assert_eq!(foreign.total.bytes(), None);
    assert_eq!(foreign.issues.other_filesystems, 1);
    assert_eq!(
        physical_dir_size(&worktree, elsewhere).measure,
        Measure::Elsewhere
    );
}

#[cfg(unix)]
#[test]
fn volume_bytes_reports_a_real_volume() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (capacity, available) = volume_bytes(tmp.path()).expect("statfs on a tempdir");
    assert!(capacity > 0);
    assert!(available <= capacity);

    let cpath = std::ffi::CString::new(tmp.path().to_string_lossy().as_bytes()).unwrap();
    // SAFETY: statvfs is zero-initializable POD and receives a valid pointer.
    let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
    assert_eq!(unsafe { libc::statvfs(cpath.as_ptr(), &mut stat) }, 0);
    fn widen(value: impl TryInto<u64>) -> Option<u64> {
        value.try_into().ok()
    }
    let block = widen(stat.f_frsize)
        .filter(|&size| size != 0)
        .or_else(|| widen(stat.f_bsize))
        .unwrap();
    assert_eq!(capacity, block * widen(stat.f_blocks).unwrap());
}
