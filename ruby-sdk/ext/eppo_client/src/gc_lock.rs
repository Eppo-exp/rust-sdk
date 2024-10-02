use magnus::Ruby;

pub struct GcLock<'a> {
    ruby: &'a Ruby,
    /// Holds `true` if GC was already disabled before acquiring the lock (so it doesn't need to be
    /// re-enabled).
    gc_was_disabled: bool,
}

impl<'a> GcLock<'a> {
    pub fn new(ruby: &'a Ruby) -> GcLock<'a> {
        GcLock {
            ruby,
            gc_was_disabled: ruby.gc_disable(),
        }
    }
}

impl<'a> Drop for GcLock<'a> {
    fn drop(&mut self) {
        if !self.gc_was_disabled {
            self.ruby.gc_enable();
        }
    }
}
