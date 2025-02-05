using System.Runtime.InteropServices;

namespace Games.Cheetah.EmbeddedServer.FFI
{
    internal static class Server
    {
        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void OnServerError([MarshalAs(UnmanagedType.LPWStr)] string message);

        [StructLayout(LayoutKind.Sequential)]
        internal struct Description
        {
            [MarshalAs(UnmanagedType.U8)] internal ulong id;

            internal unsafe fixed byte gameIp[4];
            [MarshalAs(UnmanagedType.U2)] internal ushort gamePort;

            internal unsafe fixed byte internal_grpc_ip[4];
            [MarshalAs(UnmanagedType.U2)] internal ushort internal_grpc_port;

            internal unsafe fixed byte internal_webgrpc_ip[4];
            [MarshalAs(UnmanagedType.U2)] internal ushort internal_webgrpc_port;

            internal unsafe fixed byte debug_rest_service_ip[4];
            [MarshalAs(UnmanagedType.U2)] internal ushort debug_rest_service_port;
        }

        [StructLayout(LayoutKind.Sequential)]
        internal struct BindSocket
        {
            internal unsafe fixed byte bindAddress[4];
            [MarshalAs(UnmanagedType.U2)] internal ushort port;
        }


        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "run_new_server")]
        internal static extern bool RunNewServer(ref Description description, OnServerError onServerError,
            ref BindSocket internalGrpcSocket,
            ref BindSocket internalWebGrpcSocket,
            ref BindSocket debugRestServiceSocket,
            ref BindSocket gameUdpSocket);

        [DllImport(Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "destroy_server")]
        internal static extern bool DestroyServer(ulong serverId);
    }
}