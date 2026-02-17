use uuid::Uuid;

use crate::stig::Stig;

/// A group of stig wrappers.
/// Seperated into two lists internally, stigs that have been pinned
/// and stigs that are not pinned.
/// Pinned meaning show up first in the program.
#[derive(Debug, Clone)]
pub struct SGroup {
    pinned: Vec<Box<StigWrapper>>,
    not_pinned: Vec<Box<StigWrapper>>,
}

/// A wrapper around a stig, which includes a unique identifier, and why it is
/// pinned, if it all.
#[derive(Debug, Clone)]
pub struct StigWrapper {
    pub stig: Box<Stig>,
    pub uuid: Uuid,
    pub pinned: Pinned,
}

/// Reasons a stig can be pinned.
#[derive(Debug, Clone)]
pub enum Pinned {
    ByCmd,
    ByUser,
    Not,
}

impl SGroup {
    pub fn new() -> Self {
        Self {
            pinned: Vec::new(),
            not_pinned: Vec::new(),
        }
    }

    /// Get all stigs. First pinned, then unpinned.
    pub fn get_all(&self) -> Vec<Box<StigWrapper>> {
        let mut all = self.pinned.clone();
        all.append(&mut self.not_pinned.clone());

        all
    }

    /// Sorts given the title of the stig.
    pub fn sort_by_version(&mut self) {
        self.pinned
            .sort_unstable_by(|a, b| a.stig.version.cmp(&b.stig.version));

        self.not_pinned
            .sort_unstable_by(|a, b| a.stig.version.cmp(&b.stig.version));
    }

    /// Get a stig given its uuid.
    pub fn get_by_uuid(&self, id: Uuid) -> Option<Box<StigWrapper>> {
        for stig_wrapper in self.pinned.iter() {
            if stig_wrapper.uuid == id {
                return Some(stig_wrapper.clone());
            }
        }

        for stig_wrapper in self.not_pinned.iter() {
            if stig_wrapper.uuid == id {
                return Some(stig_wrapper.clone());
            }
        }

        None
    }

    /// Add an unpinned stig to the group.
    pub fn add(&mut self, stig: Box<Stig>) {
        self.not_pinned.push(Box::new(StigWrapper {
            stig: stig,
            uuid: Uuid::new_v4(),
            pinned: Pinned::Not,
        }));
    }

    /// Add a vector of unpinned stigs to the group.
    /// Not used at the moment, but may replace set_group.
    pub fn add_group(&mut self, mut stigs: Vec<Box<Stig>>) {
        self.not_pinned.extend(stigs.drain(..).map(|stig| {
            Box::new(StigWrapper {
                stig: stig,
                uuid: Uuid::new_v4(),
                pinned: Pinned::Not,
            })
        }));
    }

    /// Set the internal group to these unpinned stigs.
    pub fn set_group(&mut self, mut stigs: Vec<Box<Stig>>) {
        self.not_pinned = stigs
            .drain(..)
            .map(|stig| {
                Box::new(StigWrapper {
                    stig: stig,
                    uuid: Uuid::new_v4(),
                    pinned: Pinned::Not,
                })
            })
            .collect();
    }

    /// Pin a stig given a uuid and a reason.
    pub fn pin(&mut self, id: Uuid, reason: Pinned) {
        let mut index = None;

        for (i, stig_wrapper) in self.not_pinned.iter().enumerate() {
            if stig_wrapper.uuid == id {
                index = Some(i);
                break;
            }
        }

        if let Some(index) = index {
            let mut stig_wrapper = self.not_pinned.swap_remove(index);
            stig_wrapper.pinned = reason;

            self.pinned.push(stig_wrapper);
        }
    }

    /// Unpin a stig given a uuid.
    pub fn unpin(&mut self, id: Uuid) {
        let mut index = None;

        for (i, stig_wrapper) in self.pinned.iter().enumerate() {
            if stig_wrapper.uuid == id {
                index = Some(i);
                break;
            }
        }

        if let Some(index) = index {
            let mut stig_wrapper = self.pinned.swap_remove(index);
            stig_wrapper.pinned = Pinned::Not;

            self.not_pinned.push(stig_wrapper);
        }
    }

    /// Unpin all stigs that were pinned by a cmd prompt.
    pub fn unpin_all_from_cmd(&mut self) {
        if self.pinned.len() == 0 {
            return;
        }

        for stig_wrapper in self.pinned.iter_mut() {
            if let Pinned::ByCmd = stig_wrapper.pinned {
                stig_wrapper.pinned = Pinned::Not;
                self.not_pinned.push(stig_wrapper.clone());
            }
        }

        let mut i: usize = 0;
        while i < self.pinned.len() {
            if let Pinned::Not = self.pinned[i].pinned {
                self.pinned.swap_remove(i);
            } else {
                i += 1;
            }
        }
    }

    /// Get the first stig stord internally.
    pub fn first(&self) -> Box<StigWrapper> {
        if self.pinned.len() != 0 {
            self.pinned[0].clone()
        } else {
            self.not_pinned[0].clone()
        }
    }

    /// Request the next stig in the list with a given uuid. If the stig
    /// is at the end of the list, wrap around to the front and get that one.
    /// Handles going from pinned -> not pinned -> back to pinned.
    pub fn get_next_wrapping(&self, id: Uuid) -> Option<Box<StigWrapper>> {
        let mut index = None;

        for (i, stig_wrapper) in self.pinned.iter().enumerate() {
            if stig_wrapper.uuid == id {
                index = Some(i);
                break;
            }
        }

        if let Some(mut index) = index {
            index += 1;
            if index >= self.pinned.len() {
                // Go into not pinned sigs.
                if self.not_pinned.len() != 0 {
                    return Some(self.not_pinned[0].clone());
                // Go into the pinned stig list.
                } else {
                    return Some(self.pinned[0].clone());
                }
            }

            return Some(self.pinned[index].clone());
        }

        for (i, stig_wrapper) in self.not_pinned.iter().enumerate() {
            if stig_wrapper.uuid == id {
                index = Some(i);
                break;
            }
        }

        if let Some(mut index) = index {
            index += 1;
            if index >= self.not_pinned.len() {
                // Go into the pinned stig list.
                if self.pinned.len() != 0 {
                    return Some(self.pinned[0].clone());
                // Go into not pinned sigs.
                } else {
                    return Some(self.not_pinned[0].clone());
                }
            }

            return Some(self.not_pinned[index].clone());
        }

        None
    }
}
