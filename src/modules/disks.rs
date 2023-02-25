use nix::sys::inotify::{AddWatchFlags, InitFlags, Inotify, WatchDescriptor};
use std::{collections::HashMap, ffi::OsStr, path::PathBuf, process::exit};

const MEDIA_DIR: &str = concat!("/run/media/", env!("USER"));

const RETRY_COUNT: i64 = 10;

pub struct DiskMon {
    notifier: Inotify,
    mount_disc: WatchDescriptor, // mount discriptors
    drive_disc: WatchDescriptor, // drives folder discriptor
    extern_drives: HashMap<String, Option<String>>,
}

impl DiskMon {
    pub fn new() -> Self {
        let notifier = Inotify::init(InitFlags::empty()).unwrap();
        let drive_disc = notifier
            .add_watch("/dev/", AddWatchFlags::IN_CREATE | AddWatchFlags::IN_DELETE)
            .unwrap_or_else(|e| {
                eprintln!("Failed to watch for devices {:?}", e);
                exit(1);
            });
        let mount_disc = notifier
            .add_watch(
                MEDIA_DIR,
                AddWatchFlags::IN_CREATE | AddWatchFlags::IN_DELETE,
            )
            .unwrap_or_else(|e| {
                eprintln!("Failed to watch for devices {:?}", e);
                exit(1);
            });
        Self {
            notifier,
            mount_disc,
            drive_disc,
            extern_drives: HashMap::new(),
        }
    }
    /// if a mount directory is created in /run/media/$USER/ means that drive is mounted, this function will map that mount point to that drive, lly for drive removal
    fn handle_mounts(&mut self, name: &OsStr, action: AddWatchFlags) {
        if !(action & AddWatchFlags::IN_CREATE).is_empty() {
            let mut retry = RETRY_COUNT;
            // it is likely that eventhough directory is created its not yet mounted, so sleep for a while
            std::thread::sleep(std::time::Duration::from_millis(100));
            while retry > 0 {
                // eventhough directory is created it may not be mounted at this point, wait for some time and recheck
                let mounts = mountinfo::MountInfo::new().unwrap();
                if let Some(m_point) = mounts
                    .mounting_points
                    .iter()
                    .find(|m_point| m_point.path.ends_with(name))
                {
                    let drive = PathBuf::from(&m_point.what);
                    if let Some(handle) = self
                        .extern_drives
                        .get_mut(drive.file_name().unwrap().to_str().unwrap())
                    {
                        *handle = Some(m_point.path.to_string_lossy().into_owned());
                    } else {
                        eprintln!(
                            "Failed to find drive {:?} in collection {:?}",
                            drive, self.extern_drives
                        );
                    }
                    break;
                } else {
                    eprintln!(
                        "failed to find mount for {:?}, may be its not yet mounted",
                        name
                    );
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    retry -= 1;
                }
            }
        } else if !(action & AddWatchFlags::IN_DELETE).is_empty() {
            for (_, val) in self.extern_drives.iter_mut() {
                if let Some(mount_point) = val {
                    let mount_path = PathBuf::from(mount_point.as_str());
                    if mount_path.file_name() == Some(name) {
                        *val = None;
                    }
                }
            }
        } else {
            unreachable!("Only mount directory create and delete are watching");
        }
    }
    /// if a drive is inserted this will check and insert to extern_drives set
    /// returns Some(()) if it finds valid drive else None
    fn handle_drives(&mut self, os_name: &OsStr, mask: AddWatchFlags) -> Result<(), ()> {
        let name_ref = os_name
            .to_str()
            .expect(&format!("Failed to convert {:?} to str", os_name));
        if let Some(last_char) = name_ref.chars().next_back() {
            // check if the last char is digit, because generally names are in the form of /dev/sd*[1-9]
            if !(last_char.is_digit(10) && name_ref.starts_with("sd")) {
                return Err(());
            }
            'check: {
                if !(mask & AddWatchFlags::IN_CREATE).is_empty() {
                    if let Some(name) = os_name.clone().to_str() {
                        self.extern_drives.insert(String::from(name), None);
                        break 'check;
                    }
                    eprintln!("Failed insert Disk {:?}", os_name);
                } else if !(mask & AddWatchFlags::IN_DELETE).is_empty() {
                    self.extern_drives.remove(name_ref);
                    break 'check;
                } else {
                    eprintln!("Uncaught event {:?} for {:?}", mask, os_name);
                }
            }
        }
        Ok(())
    }

    pub fn listen(&mut self) -> anyhow::Result<()> {
        loop {
            self.notifier.read_events()?.iter().for_each(|event| {
                if let Some(os_name) = &event.name {
                    // if its from mount directory
                    if event.wd == self.mount_disc {
                        self.handle_mounts(&os_name, event.mask);
                    } else if event.wd == self.drive_disc {
                        if let Err(_) = self.handle_drives(os_name, event.mask) {
                            return;
                        }
                    } else {
                        unreachable!();
                    }
                    if self.extern_drives.len() > 0 {
                        crate::print(&Some(&self.extern_drives));
                    } else {
                        crate::print::<()>(&None);
                    }
                    return;
                }
                eprintln!("Invalid disk name {:?}", event.name);
            })
        }
    }
}
