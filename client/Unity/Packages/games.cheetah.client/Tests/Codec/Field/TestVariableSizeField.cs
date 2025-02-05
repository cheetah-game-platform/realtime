using Games.Cheetah.Client.Codec;
using NUnit.Framework;

namespace Games.Cheetah.Client.Tests.Codec.Field
{
    public class TestVariableSizeField : AbstractFieldTest<TestVariableSizeField.Structure>
    {
        [GenerateCodec]
        public struct Structure
        {
            [VariableSizeCodec] public uint uintValue;
            [VariableSizeCodec] public ulong ulongValue;
            [VariableSizeCodec] public int intValue;
            [VariableSizeCodec] public long longValue;
        }

        protected override Structure GetSource()
        {
            return new Structure
            {
                intValue = int.MaxValue,
                longValue = long.MaxValue,
                uintValue = uint.MaxValue,
                ulongValue = ulong.MaxValue
            };
        }

        protected override void CheckResult(Structure source, Structure result)
        {
            Assert.AreEqual(source.uintValue, result.uintValue);
            Assert.AreEqual(source.ulongValue, result.ulongValue);
            Assert.AreEqual(source.intValue, result.intValue);
            Assert.AreEqual(source.longValue, result.longValue);
        }
    }
}