#if UNITY_64
using AOT;
using UnityEngine;
#else
using Serilog;
#endif
using System;
using System.Net;
using Cheetah.Matches.Realtime.EmbeddedServer.FFI;
using Cheetah.Matches.Realtime.EmbeddedServer.Impl;

#nullable enable
namespace Cheetah.Matches.Realtime.EmbeddedServer.API
{
    /// <summary>
    /// Класс для запуска встроенного realtime сервера, в основном используется для тестов.
    /// Однако может использоваться и для production целей.
    ///
    /// Порядок использования:
    ///  - создаем экземпляр сервера (можно один сервер, на один тест)
    ///  - создаем комнату
    ///  - создаем необходимое количество пользователей в команте
    ///  - соединяемся с сервером от имени клиента(ов) для тестирования
    ///  - удаляем сервер для освобождения ресурсов
    /// </summary>
    public class EmbeddedServer
    {
        private readonly Server.Description description;
        private static string? errorMessage;

        public EmbeddedServer(IPAddress bindAddress)
        {
            unsafe
            {
                var bindFFIAddress = new Server.BindAddress();
                var addressBytes = bindAddress.GetAddressBytes();

                bindFFIAddress.bindAddress[0] = addressBytes[0];
                bindFFIAddress.bindAddress[1] = addressBytes[1];
                bindFFIAddress.bindAddress[2] = addressBytes[2];
                bindFFIAddress.bindAddress[3] = addressBytes[3];

                if (!Server.RunNewServer(ref description, OnError, ref bindFFIAddress))
                {
                    throw new Exception("Cannot run embedded server. " + errorMessage);
                }
            }
        }

        public ServerRoom CreateRoom()
        {
            ulong roomId = 0;
            if (!Room.CreateRoom(description.id, ref roomId, OnError))
            {
                throw new Exception("Cannot create room. " + errorMessage);
            }

            return new ServerRoomImpl(description, roomId);
        }

        public void Destroy()
        {
            if (!Server.DestroyServer(description.id))
            {
                throw new Exception("Embedded server not found");
            }
        }

        public string GetGameIp()
        {
            unsafe
            {
                return
                    $"{description.gameIp[0]}.{description.gameIp[1]}.{description.gameIp[2]}.{description.gameIp[3]}";
            }
        }

        public string GetAdminGrpcIp()
        {
            unsafe
            {
                return
                    $"{description.admin_grpc_ip[0]}.{description.admin_grpc_ip[1]}.{description.admin_grpc_ip[2]}.{description.admin_grpc_ip[3]}";
            }
        }

        public string GetInternalGrpcIp()
        {
            unsafe
            {
                return
                    $"{description.internal_grpc_ip[0]}.{description.internal_grpc_ip[1]}.{description.internal_grpc_ip[2]}.{description.internal_grpc_ip[3]}";
            }
        }

        public uint GetGamePort()
        {
            return description.gamePort;
        }

        public uint GetAdminGrpcPort()
        {
            return description.admin_grpc_port;
        }

        public uint GetInternalGrpcPort()
        {
            return description.internal_grpc_port;
        }

#if UNITY_64
        [MonoPInvokeCallback(typeof(Server.OnServerError))]
#endif
        private static void OnError(string? message)
        {
            errorMessage = message;
        }


        public static void InitLogger(EmeddedServerLogLevel emeddedServerLogLevel)
        {
            Logger.InitLogger();
            Logger.SetMaxLogLevel(emeddedServerLogLevel);
        }

        public static void ShowCurrentLogs()
        {
            Logger.CollectLogs(ShowLog);
        }

#if UNITY_64
        [MonoPInvokeCallback(typeof(FFI.Logger.LogCollector))]
        private static void ShowLog(EmeddedServerLogLevel level, string log)
        {
            switch (level)
            {
                case EmeddedServerLogLevel.Info:
                    Debug.Log(log);
                    break;
                case EmeddedServerLogLevel.Warn:
                    Debug.LogWarning(log);
                    break;
                case EmeddedServerLogLevel.Error:
                    Debug.LogError(log);
                    break;
                default:
                    throw new ArgumentOutOfRangeException(nameof(level), level, null);
            }
        }
#else
        private static void ShowLog(EmeddedServerLogLevel level, string log)
        {
            switch (level)
            {
                case EmeddedServerLogLevel.Info:
                    Log.Information(log);
                    break;
                case EmeddedServerLogLevel.Warn:
                    Log.Warning(log);
                    break;
                case EmeddedServerLogLevel.Error:
                    Log.Error(log);
                    break;
                default:
                    throw new ArgumentOutOfRangeException(nameof(level), level, null);
            }
        }
#endif
    }
}