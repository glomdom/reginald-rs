use log::trace;
use uefi::{
    Guid, Handle,
    boot::{self, OpenProtocolAttributes, OpenProtocolParams, ScopedProtocol, SearchType},
    proto::Protocol,
};

pub fn get_handle_from_guid(guid: Guid) -> Handle {
    trace!("GET handle {}", guid);

    return *boot::locate_handle_buffer(SearchType::ByProtocol(&guid))
        .expect("failed to locate handle with provided guid")
        .first()
        .expect("protocol not found in handle array");
}

pub fn get_protocol_from_handle<P: Protocol>(handle: Handle) -> ScopedProtocol<P> {
    trace!("OPEN protocol {}", P::GUID);

    boot::open_protocol_exclusive::<P>(handle)
        .expect("failed to open protocol with provided handle")
}

/// Shared version of `get_protocol_from_handle`. Allows the
/// protocol to not be dropped when moving out of scope.
pub fn get_shared_protocol<P: Protocol>() -> ScopedProtocol<P> {
    trace!("SHARED protocol {}", P::GUID);

    let handle = get_handle_from_guid(P::GUID);

    unsafe {
        boot::open_protocol::<P>(
            OpenProtocolParams {
                handle: handle,
                agent: handle,
                controller: None,
            },
            OpenProtocolAttributes::GetProtocol,
        )
    }
    .expect("failed to open protocol")
}

/// Gets the protocol using a handle.
/// The handle is located with the protocol's GUID.
/// If a protocol that requires another handle is needed, use `get_protocol_from_handle`
#[allow(dead_code)]
pub fn get_protocol<P: Protocol>() -> ScopedProtocol<P> {
    trace!("GET protocol {}", P::GUID);

    let handle = get_handle_from_guid(P::GUID);

    get_protocol_from_handle(handle)
}
