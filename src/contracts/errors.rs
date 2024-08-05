use alloy::{contract::Error, transports::RpcError};

pub struct ErrorLogger;

impl ErrorLogger {
    pub fn log_contract_error(error: Error) {
        match error {
            Error::UnknownFunction(function) => {
                log::error!("Unknown function: function {} does not exist", function);
            }
            Error::UnknownSelector(selector) => {
                log::error!(
                    "Unknown function: function with selector {} does not exist",
                    selector
                );
            }
            Error::NotADeploymentTransaction => {
                log::error!("Transaction is not a deployment transaction");
            }
            Error::ContractNotDeployed => {
                log::error!("Contract is not deployed");
            }
            Error::AbiError(e) => {
                log::error!("An error occurred ABI encoding or decoding: {}", e);
            }
            Error::TransportError(error) => {
                if let RpcError::ErrorResp(payload) = error {
                    let error = payload
                        .as_decoded_error::<crate::contracts::OracleRegistry::OracleRegistryErrors>(
                            false,
                        )
                        .unwrap();

                    if let crate::contracts::OracleRegistry::OracleRegistryErrors::AlreadyRegistered(
                        e,
                    ) = error
                    {
                        log::info!("Already registered: {}", e._0);
                    } else {
                        log::error!(
                            "Transport error: {:?}",
                            String::from_utf8(error.abi_encode())
                        );
                    }
                } else {
                    log::error!("Unknown transport error: {:#?}", error);
                }
            }
        }
    }
}
