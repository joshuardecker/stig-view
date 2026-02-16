use uuid::Uuid;

use crate::stig::Stig;

#[derive(Debug, Clone)]
pub struct SGroup {
    pinned: Vec<Box<StigWrapper>>,
    not_pinned: Vec<Box<StigWrapper>>,
}

#[derive(Debug, Clone)]
pub struct StigWrapper {
    pub stig: Box<Stig>,
    pub uuid: Uuid,
    pub pinned: Pinned,
}

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

    pub fn get_all(&self) -> Vec<Box<StigWrapper>> {
        let mut all = self.pinned.clone();
        all.append(&mut self.not_pinned.clone());

        all
    }

    pub fn sort_by_version(&mut self) {
        self.pinned
            .sort_unstable_by(|a, b| a.stig.version.cmp(&b.stig.version));

        self.not_pinned
            .sort_unstable_by(|a, b| a.stig.version.cmp(&b.stig.version));
    }

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

    pub fn add(&mut self, stig: Box<Stig>) {
        self.not_pinned.push(Box::new(StigWrapper {
            stig: stig,
            uuid: Uuid::new_v4(),
            pinned: Pinned::Not,
        }));
    }

    pub fn add_group(&mut self, mut stigs: Vec<Box<Stig>>) {
        self.not_pinned.extend(stigs.drain(..).map(|stig| {
            Box::new(StigWrapper {
                stig: stig,
                uuid: Uuid::new_v4(),
                pinned: Pinned::Not,
            })
        }));
    }

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

    pub fn first(&self) -> Box<StigWrapper> {
        if self.pinned.len() != 0 {
            self.pinned[0].clone()
        } else {
            self.not_pinned[0].clone()
        }
    }

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
                index = 0;
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
                index = 0;
            }

            return Some(self.not_pinned[index].clone());
        }

        None
    }
}
