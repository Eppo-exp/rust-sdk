use std::sync::{Mutex, RwLock};

use pyo3::{exceptions::PyException, prelude::*};

use crate::{client::EppoClient, client_config::ClientConfig};

// TODO: use `pyo3::sync::GILProtected` instead?
static CLIENT_INSTANCE: RwLock<Option<Py<EppoClient>>> = RwLock::new(None);

/// Initializes a global Eppo client instance.
///
/// This method should be called once on application startup.
/// If invoked more than once, it will re-initialize the global client instance.
/// Use the :func:`EppoClient.get_instance()` method to access the client instance.
///
/// :param config: client configuration containing the API Key
/// :type config: Config
#[pyfunction]
pub fn init(config: Bound<ClientConfig>) -> PyResult<Py<EppoClient>> {
    initialize_pyo3_log();

    let py = config.py();

    let client = Bound::new(py, EppoClient::new(py, &*config.borrow())?)?.unbind();

    // minimizing the scope of holding the write lock
    let existing = {
        let client = Py::clone_ref(&client, py);

        let mut instance = CLIENT_INSTANCE.write().map_err(|err| {
            // This should normally never happen as it signifies that another thread
            // panicked while holding the lock.
            PyException::new_err(format!("failed to acquire writer lock: {err}"))
        })?;
        std::mem::replace(&mut *instance, Some(client))
    };
    if let Some(existing) = existing {
        existing.get().shutdown();
        existing.drop_ref(py);
    }

    Ok(client)
}

/// Used to access an initialized client instance.
///
/// Use this method to get a client instance for assigning variants.
/// This method may only be called after invocation of :func:`eppo_client.init()`, otherwise it
/// throws an exception.
///
/// :return: a shared client instance
/// :rtype: EppoClient
#[pyfunction]
pub fn get_instance(py: Python) -> PyResult<Py<EppoClient>> {
    let instance = CLIENT_INSTANCE.read().map_err(|err| {
        // This should normally never happen as it signifies that another thread
        // panicked while holding the lock.
        PyException::new_err(format!("failed to acquire reader lock: {err}"))
    })?;
    if let Some(existing) = &*instance {
        Ok(Py::clone_ref(existing, py))
    } else {
        Err(PyException::new_err(
            "init() must be called before get_instance()",
        ))
    }
}

/// Initialize `pyo3_log` crate connecting Rust's `log` to Python's `logger`.
///
/// If called multiple times, resets the pyo3_log cache.
fn initialize_pyo3_log() {
    static LOG_RESET_HANDLE: Mutex<Option<pyo3_log::ResetHandle>> = Mutex::new(None);
    {
        if let Ok(mut reset_handle) = LOG_RESET_HANDLE.lock() {
            if let Some(previous_handle) = &mut *reset_handle {
                // There's a previous handle. Logging is already initialized, but we reset
                // caches.
                previous_handle.reset();
            } else {
                if let Ok(new_handle) = pyo3_log::try_init() {
                    *reset_handle = Some(new_handle);
                } else {
                    // This should not happen as initialization error signals that we already
                    // initialized logging. (In which case, `LOG_RESET_HANDLE` should contain
                    // `Some()`.)
                    debug_assert!(false, "tried to initialize pyo3_log second time");
                }
            }
        } else {
            // This should normally never happen as it shows that another thread has panicked
            // while holding `LOG_RESET_HANDLE`.
            //
            // That's probably not sever enough to throw an exception into user's code.
            debug_assert!(false, "failed to acquire LOG_RESET_HANDLE lock");
        }
    }
}
