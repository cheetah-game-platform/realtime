using Games.Cheetah.Client.Codec;
using NUnit.Framework;

namespace Games.Cheetah.Client.Tests.Codec.Field
{
    public class TestEnumField : AbstractFieldTest<TestEnumField.Structure>
    {
        public enum EnumTest
        {
            A,
            B,
            C,
            D
        }

        [GenerateCodec]
        public struct Structure
        {
            public EnumTest enumValue;
        }

        protected override Structure GetSource()
        {
            return new Structure
            {
                enumValue = EnumTest.D
            };
        }

        protected override void CheckResult(Structure source, Structure result)
        {
            Assert.AreEqual(source.enumValue, result.enumValue);
        }
    }
}