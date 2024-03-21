#![allow(dead_code)]

use concurrent_queue::ConcurrentQueue;

#[derive(Debug)]
pub struct QList {
    pub qlist: Vec<ConcurrentQueue<Vec<f32>>>,
    pub length: u16,
    index: usize,
}

impl QList {
    pub fn new() -> Self {
        let q: Vec<ConcurrentQueue<Vec<f32>>> = Vec::new();
        Self {
            qlist: q,
            length: 0,
            index: 0,
        }
    }

    pub fn initialize(&mut self) {
        self.qlist.push(ConcurrentQueue::<Vec<f32>>::unbounded());
        self.length += 1;
    }

    pub fn put_frame(&mut self, frame: Vec<f32>) {
        self.qlist[self.index].push(frame).unwrap()
    }

    pub fn get_frame(&mut self, index: usize) -> Vec<f32> {
        self.qlist[index].pop().unwrap()
    }

    pub fn is_empty_at_index(&self, index: usize) -> bool {
        self.qlist[index].is_empty()
    }

    pub fn get_next_empty_queue(&mut self) {
        let mut counter = 0;
        while counter < self.length {
            if self.is_empty_at_index(self.index) {
                break;
            }

            self.index += 1;
            self.index %= self.length as usize;
            counter += 1;

            if counter >= self.length {
                let q = ConcurrentQueue::<Vec<f32>>::unbounded();
                self.qlist.push(q);
                self.length += 1;
                self.index = (self.length - 1) as usize;
            }
        }
    }

    pub fn is_all_empty(&self) -> bool {
        self.qlist.iter().all(|x| x.is_empty())
    }
}

impl Default for QList {
    fn default() -> Self {
        let qlist: Vec<ConcurrentQueue<Vec<f32>>> = vec![ConcurrentQueue::<Vec<f32>>::unbounded()];
        Self {
            qlist,
            length: 1,
            index: 0,
        }
    }
}

impl Clone for QList {
    fn clone(&self) -> Self {
        let mut qclone: Vec<ConcurrentQueue<Vec<f32>>> = Vec::new();

        for q in &self.qlist {
            let cloned_queue = ConcurrentQueue::<Vec<f32>>::unbounded();
            for item in q.try_iter() {
                cloned_queue.push(item.clone()).unwrap();
            }
            qclone.push(cloned_queue);
        }

        Self {
            qlist: qclone,
            length: self.length,
            index: self.index,
        }
    }
}

unsafe impl Send for QList {}
unsafe impl Sync for QList {}
