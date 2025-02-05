using System;
using System.Net;
using System.Threading;
using System.Threading.Tasks;
using Games.Cheetah.Client;
using Games.Cheetah.Client.Codec;
using Games.Cheetah.Client.Types.Network;
using Games.Cheetah.EmbeddedServer.API;
using Games.Cheetah.Realtime.GRPC;
using NUnit.Framework;

namespace Games.Cheetah.EmbeddedServer.Tests
{
    public class EmbeddedServerTests
    {
        [Test]
        public void Test()
        {
            Task.Run(async () =>
            {
                API.EmbeddedServer.InitLogger(EmeddedServerLogLevel.Warn);

                var server = new API.EmbeddedServer(IPAddress.Loopback);
                var grpcClient = server.CreateGrpcClient();
                var room = await grpcClient.CreateRoomAsync(new RoomTemplate());
                var member = await grpcClient.CreateMemberAsync(new CreateMemberRequest
                {
                    RoomId = room.RoomId,
                    User = new UserTemplate
                    {
                        Groups = 0b000111
                    }
                });


                var client = new NetworkClient(
                    0,
                    server.GetUdpGameHost(),
                    server.GetUdpGamePort(),
                    member.UserId,
                    room.RoomId,
                    member.PrivateKey.ToByteArray(),
                    new CodecRegistryBuilder().Build(),
                    10);
                client.DisableClientLog();
                client.Update();

                // небольшая пауза для обмена сетевыми пакетами
                Thread.Sleep(TimeSpan.FromSeconds(1));

                // проверяем факт соединения
                Assert.AreEqual(client.GetConnectionStatus(), ConnectionStatus.Connected);

                // останавливаем сервер
                server.Destroy();

                // сервер остановлен - выжидаем окончания timeout на клиентские команды
                Thread.Sleep(TimeSpan.FromSeconds(11));
                Assert.AreNotEqual(client.GetConnectionStatus(), ConnectionStatus.Connected);

                API.EmbeddedServer.ShowCurrentLogs();
            });
        }
    }
}