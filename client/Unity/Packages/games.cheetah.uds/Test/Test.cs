using System;
using System.Net;
using System.Threading.Tasks;
using Games.Cheetah.Client;
using Games.Cheetah.Client.Codec;
using Games.Cheetah.Client.Types.Network;
using Games.Cheetah.EmbeddedServer.API;
using Games.Cheetah.Realtime.GRPC;
using Games.Cheetah.UDS.API;
using NUnit.Framework;

namespace Games.Cheetah.UDS.Tests.Test
{
    public class Test
    {
        private ulong createdRoomId;
        private NetworkClient cheetahClient;
        private EmbeddedServer.API.EmbeddedServer server;
        private UDSPlugin plugin;


        [SetUp]
        public void SetUp()
        {
            EmbeddedServer.API.EmbeddedServer.InitLogger(EmeddedServerLogLevel.Warn);
            server = new EmbeddedServer.API.EmbeddedServer(IPAddress.Loopback);
            plugin = new UDSPlugin(
                server.GetInternalWebGrpcUri(),
                server.GetUdpGameHost(),
                server.GetUdpGamePort(),
                OnRoomCreated,
                OnRoomDeleted,
                new CodecRegistryBuilder().Build());
        }


        [Test]
        public void ShouldCreatePluginWhenCreateRoom()
        {
            Task.Run(async () =>
            {
                var grpcClient = server.CreateGrpcClient();
                var room = await grpcClient.CreateRoomAsync(new RoomTemplate());
                await Task.Delay(TimeSpan.FromSeconds(2));

                await plugin.OnUpdate();
                // даем время cheetahClient перейти в состояние connected
                await Task.Delay(TimeSpan.FromSeconds(2));

                Assert.AreEqual(room.RoomId, createdRoomId);
                Assert.AreEqual(cheetahClient.GetConnectionStatus(), ConnectionStatus.Connected);
            }).GetAwaiter().GetResult();
        }


        private void OnRoomDeleted(ulong roomId)
        {
            throw new NotImplementedException();
        }

        private void OnRoomCreated(ulong roomId,
            RealtimeServerManagementService.RealtimeServerManagementServiceClient internalClient,
            NetworkClient cheetahClient)
        {
            createdRoomId = roomId;
            this.cheetahClient = cheetahClient;
        }
    }
}