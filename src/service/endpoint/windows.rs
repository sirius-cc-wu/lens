use std::{ffi::c_void, io, mem, ptr};

use tokio::net::windows::named_pipe::{
    ClientOptions, NamedPipeClient, NamedPipeServer, ServerOptions,
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, LocalFree, ERROR_ACCESS_DENIED, HANDLE},
    Security::{
        Authorization::{
            ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW,
            SDDL_REVISION_1,
        },
        GetTokenInformation, TokenUser, PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES, TOKEN_QUERY,
        TOKEN_USER,
    },
    System::Threading::{GetCurrentProcess, OpenProcessToken},
};

use super::EndpointError;

pub(crate) type ClientConnection = NamedPipeClient;
pub(crate) type ServerConnection = NamedPipeServer;

pub(crate) struct Listener {
    pipe_name: String,
    user_sid: String,
    inner: NamedPipeServer,
}

impl Listener {
    pub(crate) async fn accept(&mut self) -> Result<ServerConnection, EndpointError> {
        self.inner
            .connect()
            .await
            .map_err(|source| EndpointError::io("Could not accept a Lens command", source))?;
        let next = create_server(&self.pipe_name, &self.user_sid, false)?;
        Ok(mem::replace(&mut self.inner, next))
    }
}

pub(crate) async fn connect() -> Result<ClientConnection, EndpointError> {
    let sid = current_user_sid()?;
    ClientOptions::new()
        .open(pipe_name(&sid))
        .map_err(|source| {
            EndpointError::io("Could not connect to the Lens background service", source)
        })
}

pub(crate) fn claim() -> Result<Listener, EndpointError> {
    let user_sid = current_user_sid()?;
    let pipe_name = pipe_name(&user_sid);
    let inner = create_server(&pipe_name, &user_sid, true)?;
    Ok(Listener {
        pipe_name,
        user_sid,
        inner,
    })
}

pub(crate) fn authorize(_connection: &ServerConnection) -> Result<(), EndpointError> {
    // The protected named-pipe ACL is evaluated by the kernel before a
    // client connection can reach this server instance.
    Ok(())
}

fn create_server(
    pipe_name: &str,
    user_sid: &str,
    first_instance: bool,
) -> Result<NamedPipeServer, EndpointError> {
    let descriptor = SecurityDescriptor::new(&security_descriptor_sddl(user_sid))?;
    let mut attributes = SECURITY_ATTRIBUTES {
        nLength: mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor.0,
        bInheritHandle: 0,
    };
    let mut options = ServerOptions::new();
    options
        .first_pipe_instance(first_instance)
        .reject_remote_clients(true);
    // SAFETY: attributes and its owned descriptor remain valid for the
    // complete create call, and Tokio does not retain their pointers.
    let result = unsafe {
        options.create_with_security_attributes_raw(
            pipe_name,
            &mut attributes as *mut SECURITY_ATTRIBUTES as *mut c_void,
        )
    };
    result.map_err(|source| {
        if first_instance && source.raw_os_error() == Some(ERROR_ACCESS_DENIED as i32) {
            EndpointError::AlreadyOwned
        } else {
            EndpointError::io("Could not claim the Lens command endpoint", source)
        }
    })
}

fn pipe_name(user_sid: &str) -> String {
    format!(r"\\.\pipe\lens-{user_sid}-v1")
}

fn security_descriptor_sddl(user_sid: &str) -> String {
    format!("D:P(A;;GA;;;SY)(A;;GA;;;{user_sid})")
}

fn current_user_sid() -> Result<String, EndpointError> {
    // SAFETY: all Windows handles and buffers are checked before use and
    // released by their matching Win32 functions.
    unsafe {
        let mut raw_token: HANDLE = 0;
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut raw_token) == 0 {
            return Err(EndpointError::io(
                "Could not open the current user token",
                io::Error::last_os_error(),
            ));
        }
        let token = OwnedHandle(raw_token);
        let mut size = 0;
        GetTokenInformation(token.0, TokenUser, ptr::null_mut(), 0, &mut size);
        if size == 0 {
            return Err(EndpointError::io(
                "Could not size the current user identity",
                io::Error::last_os_error(),
            ));
        }
        let mut buffer = vec![0u8; size as usize];
        if GetTokenInformation(
            token.0,
            TokenUser,
            buffer.as_mut_ptr().cast(),
            size,
            &mut size,
        ) == 0
        {
            return Err(EndpointError::io(
                "Could not read the current user identity",
                io::Error::last_os_error(),
            ));
        }
        let token_user = &*(buffer.as_ptr() as *const TOKEN_USER);
        let mut sid_text = ptr::null_mut();
        if ConvertSidToStringSidW(token_user.User.Sid, &mut sid_text) == 0 {
            return Err(EndpointError::io(
                "Could not format the current user identity",
                io::Error::last_os_error(),
            ));
        }
        let length = (0..).take_while(|&index| *sid_text.add(index) != 0).count();
        let sid = String::from_utf16(std::slice::from_raw_parts(sid_text, length)).map_err(|_| {
            EndpointError::io(
                "Could not decode the current user identity",
                io::Error::new(io::ErrorKind::InvalidData, "user SID is not valid UTF-16"),
            )
        });
        LocalFree(sid_text.cast());
        sid
    }
}

struct OwnedHandle(HANDLE);

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        // SAFETY: the handle was returned by OpenProcessToken and is owned.
        unsafe {
            CloseHandle(self.0);
        }
    }
}

struct SecurityDescriptor(PSECURITY_DESCRIPTOR);

impl SecurityDescriptor {
    fn new(sddl: &str) -> Result<Self, EndpointError> {
        let mut wide = sddl.encode_utf16().collect::<Vec<_>>();
        wide.push(0);
        let mut descriptor = ptr::null_mut();
        // SAFETY: wide is null-terminated and descriptor is a valid output
        // pointer. The returned allocation is owned by this wrapper.
        if unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                wide.as_ptr(),
                SDDL_REVISION_1,
                &mut descriptor,
                ptr::null_mut(),
            )
        } == 0
        {
            return Err(EndpointError::io(
                "Could not create the Lens named-pipe access policy",
                io::Error::last_os_error(),
            ));
        }
        Ok(Self(descriptor))
    }
}

impl Drop for SecurityDescriptor {
    fn drop(&mut self) {
        // SAFETY: the descriptor was allocated by LocalAlloc through the
        // conversion function and is released exactly once here.
        unsafe {
            LocalFree(self.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        create_server, current_user_sid, pipe_name, security_descriptor_sddl, EndpointError,
    };

    #[test]
    fn current_user_pipe_policy_then_names_user_and_allows_only_user_and_system() {
        // Arrange
        let sid = current_user_sid().expect("current user SID should be readable");

        // Act
        let name = pipe_name(&sid);
        let policy = security_descriptor_sddl(&sid);

        // Assert
        assert!(name.contains(&sid));
        assert_eq!(policy, format!("D:P(A;;GA;;;SY)(A;;GA;;;{sid})"));
    }

    #[tokio::test]
    async fn second_first_pipe_instance_then_existing_listener_remains_owner() {
        // Arrange
        let sid = current_user_sid().expect("current user SID should be readable");
        let name = format!(r"\\.\pipe\lens-owner-test-{}-{}", std::process::id(), sid);
        let first =
            create_server(&name, &sid, true).expect("first pipe instance should become owner");

        // Act
        let second = create_server(&name, &sid, true);

        // Assert
        assert!(matches!(second, Err(EndpointError::AlreadyOwned)));
        drop(first);
    }
}
