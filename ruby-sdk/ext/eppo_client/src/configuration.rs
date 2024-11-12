use std::sync::Arc;

use magnus::{function, method, prelude::*, scan_args::get_kwargs, Error, RHash, RString, Ruby};

use eppo_core::{ufc::UniversalFlagConfig, Configuration as CoreConfiguration};

use crate::{gc_lock::GcLock, SDK_METADATA};

pub(crate) fn init(ruby: &Ruby) -> Result<(), Error> {
    let eppo_client = ruby.define_module("EppoClient")?;

    let configuration = eppo_client.define_class("Configuration", magnus::class::object())?;
    configuration.define_singleton_method("new", function!(Configuration::new, 1))?;
    configuration.define_method(
        "flags_configuration",
        method!(Configuration::flags_configuration, 0),
    )?;
    configuration.define_method(
        "bandits_configuration",
        method!(Configuration::bandits_configuration, 0),
    )?;

    Ok(())
}

#[derive(Debug, Clone)]
#[magnus::wrap(class = "EppoClient::Configuration", free_immediately)]
pub struct Configuration {
    inner: Arc<CoreConfiguration>,
}

impl Configuration {
    fn new(ruby: &Ruby, kw: RHash) -> Result<Configuration, Error> {
        let args = get_kwargs(kw, &["flags_configuration"], &["bandits_configuration"])?;
        let (flags_configuration,): (RString,) = args.required;
        let (bandits_configuration,): (Option<Option<RString>>,) = args.optional;
        let rest: RHash = args.splat;
        if !rest.is_empty() {
            return Err(Error::new(
                ruby.exception_arg_error(),
                format!("unexpected keyword arguments: {:?}", rest),
            ));
        }

        let inner = {
            let _gc_lock = GcLock::new(ruby);

            Arc::new(CoreConfiguration::from_server_response(
                UniversalFlagConfig::from_json(
                    SDK_METADATA,
                    unsafe {
                        // SAFETY: we have disabled GC, so the memory can't be modified concurrently.
                        flags_configuration.as_slice()
                    }
                    .to_vec(),
                )
                .map_err(|err| {
                    Error::new(
                        ruby.exception_arg_error(),
                        format!("failed to parse flags_configuration: {err:?}"),
                    )
                })?,
                bandits_configuration
                    .flatten()
                    .map(|bandits| {
                        serde_json::from_slice(unsafe {
                            // SAFETY: we have disabled GC, so the memory can't be modified concurrently.
                            bandits.as_slice()
                        })
                    })
                    .transpose()
                    .map_err(|err| {
                        Error::new(
                            ruby.exception_arg_error(),
                            format!("failed to parse bandits_configuration: {err:?}"),
                        )
                    })?,
            ))
        };

        Ok(Configuration { inner })
    }

    fn flags_configuration(ruby: &Ruby, rb_self: &Self) -> Result<RString, Error> {
        Ok(ruby.str_from_slice(rb_self.inner.flags.to_json()))
    }

    fn bandits_configuration(ruby: &Ruby, rb_self: &Self) -> Result<Option<RString>, Error> {
        let Some(bandits) = &rb_self.inner.bandits else {
            return Ok(None);
        };
        let vec = serde_json::to_vec(bandits).map_err(|err| {
            // this should never happen
            Error::new(
                ruby.exception_runtime_error(),
                format!("failed to serialize bandits configuration: {err:?}"),
            )
        })?;
        Ok(Some(ruby.str_from_slice(&vec)))
    }
}

impl From<Arc<CoreConfiguration>> for Configuration {
    fn from(inner: Arc<CoreConfiguration>) -> Configuration {
        Configuration { inner }
    }
}

impl From<Configuration> for Arc<CoreConfiguration> {
    fn from(value: Configuration) -> Arc<CoreConfiguration> {
        value.inner
    }
}
