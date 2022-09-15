using Cheetah.Matches.Realtime.Codec;
using Cheetah.Matches.Realtime.Codec.Formatter;
using Cheetah.Matches.Realtime.Types;
using UnityEngine;
using Cheetah.Matches.Realtime.Tests.Codec.Field;

// ReSharper disable once CheckNamespace
namespace Cheetah_Matches_Realtime_Tests_Codec_Field
{
		// warning warning warning warning warning
		// Code generated by Cheetah relay codec generator - DO NOT EDIT
		// warning warning warning warning warning
		public class TestCodecArrayFieldInnerCodec:Codec<Cheetah.Matches.Realtime.Tests.Codec.Field.TestCodecArrayField.Inner>
		{
			public void Decode(ref CheetahBuffer buffer, ref Cheetah.Matches.Realtime.Tests.Codec.Field.TestCodecArrayField.Inner dest)
			{
				dest.value = IntFormatter.Instance.Read(ref buffer);
			}
	
			public void  Encode(in Cheetah.Matches.Realtime.Tests.Codec.Field.TestCodecArrayField.Inner source, ref CheetahBuffer buffer)
			{
				IntFormatter.Instance.Write(source.value,ref buffer);
			}
	
	
			[RuntimeInitializeOnLoadMethod(RuntimeInitializeLoadType.SubsystemRegistration)]
			private static void OnRuntimeMethodLoad()
			{
				CodecRegistryBuilder.RegisterDefault(factory=>new TestCodecArrayFieldInnerCodec());
			}
	
		}
	
	
}
