using System;

namespace Games.Cheetah.Client.Editor.Generator
{
    ///
    /// Генератор кодека для поля структуры
    /// 
    public interface FieldCodecGenerator
    {
        public string GenerateEncode();
        public string GenerateDecode();
    }

    public interface FieldInfoAccessor
    {
        T GetCustomAttribute<T>() where T : Attribute;
        Type FieldType { get; }
        string Name { get; }
    }
}