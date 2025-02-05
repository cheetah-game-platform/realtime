using System.Linq;
using System.Threading;
using Games.Cheetah.Client.Tests.Server.Helpers;
using NUnit.Framework;

namespace Games.Cheetah.Client.Tests.Server
{
    public class DoubleFieldTest : AbstractTest
    {
        [Test]
        public void ShouldSet()
        {
                // создаем объект на первом клиенте
                var createdObject = clientA.NewObjectBuilder(777, PlayerHelper.PlayerGroup).Build();
                // изменяем значение
                clientA.Writer.SetDouble(in createdObject.ObjectId, HealFieldId, 77.99);
                // ждем отправки команды
                Thread.Sleep(500);
                // прием команды
                clientB.Update();
                // проверяем результат
                var stream = clientB.Reader.GetModifiedDoubles(HealFieldId);
                var actual = stream.SearchLast(it=>it.Item1==createdObject.ObjectId).Item2;
                Assert.AreEqual(77.99, actual);
                stream.Dispose();
        }

        [Test]
        public void ShouldIncrement()
        {
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(777, PlayerHelper.PlayerGroup).Build();
            // изменяем значение
            clientA.Writer.Increment(in createdObject.ObjectId, HealFieldId, 77.99);
            clientA.Writer.Increment(in createdObject.ObjectId, HealFieldId, 100);
            // ждем отправки команды
            Thread.Sleep(200);
            // прием команды
            clientB.Update();
            // проверяем результат
            var stream = clientB.Reader.GetModifiedDoubles(HealFieldId);
            var actual = stream.SearchLast(it=>it.Item1==createdObject.ObjectId).Item2;
            Assert.AreEqual(177.99, actual);
            stream.Dispose();
        }
    }
}