# User applications

This is the application root at /user/app for native OpenXzuff UI programs. Applications are user-space programs and must not call kernel internals directly; a future platform SDK will provide real syscall/IPC wrappers.

The app manifest header is a lightweight metadata contract. It is not yet a package installer or a .lap/.luap parser.
