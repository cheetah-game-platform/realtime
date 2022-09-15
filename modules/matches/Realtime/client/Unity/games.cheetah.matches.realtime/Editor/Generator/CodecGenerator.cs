using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using System.Text;
using Cheetah.Matches.Realtime.Editor.Generator.Fields;
using Cheetah.Matches.Realtime.Editor.Generator.Fields.Array;
using static Cheetah.Matches.Realtime.Editor.Generator.Utils;

namespace Cheetah.Matches.Realtime.Editor.Generator
{
    internal class CodecGenerator
    {
        private readonly Formatters formatters;
        private readonly string codecNamespace;
        private readonly Type type;
        private readonly string codecClassName;
        private readonly CodecsImporter codecsImporter;


        internal CodecGenerator(Formatters formatters, string codecNamespace, Type type)
        {
            this.formatters = formatters;
            this.codecNamespace = codecNamespace;
            this.type = type;
            codecClassName = GetFullName(type).Replace(type.Namespace??" ","").Replace(".","") + "Codec";
            codecsImporter = new CodecsImporter(codecClassName);
        }

        internal string Generate()
        {
            var result = new StringBuilder();
            result.Append(GenerateUsing());
            result.Append(GenerateNamespace(AddTabs(GenerateCodecClass())));
            return result.ToString();
        }

        private string GenerateCodecClass()
        {
            var result = new StringBuilder();
            result.AppendLine("// warning warning warning warning warning");
            result.AppendLine("// Code generated by Cheetah relay codec generator - DO NOT EDIT");
            result.AppendLine("// warning warning warning warning warning");
            var targetTypeFullName = GetFullName(type);
            result.AppendLine($"public class {codecClassName}:Codec<{targetTypeFullName}>");
            result.AppendLine("{");

            var decodeMethod = new StringBuilder();
            var encodeMethod = new StringBuilder();

            decodeMethod.AppendLine($"public void Decode(ref CheetahBuffer buffer, ref {targetTypeFullName} dest)");
            decodeMethod.AppendLine("{");

            encodeMethod.AppendLine($"public void  Encode(in {targetTypeFullName} source, ref CheetahBuffer buffer)");
            encodeMethod.AppendLine("{");

            var processedFields = new HashSet<string>();
            var allFields = type.GetFields().Where(f => !f.IsStatic).Select(f => f.Name).ToHashSet();
            foreach (var field in type.GetFields())
            {
                if (field.IsStatic)
                {
                    continue;
                }

                try
                {
                    var fieldInfoAccessor = new FieldInfoAccessorImpl(field);
                    var generator =
                        VariableSizeFieldGenerator.Create(formatters, fieldInfoAccessor) ??
                        FormattedPresentTypesFieldGenerator.Create(formatters, fieldInfoAccessor) ??
                        EnumFieldGenerator.Create(formatters, fieldInfoAccessor) ??
                        FixedArrayFieldGenerator.Create(formatters, fieldInfoAccessor, processedFields, allFields) ??
                        FormatterArrayFieldGenerator.Create(formatters, fieldInfoAccessor, processedFields, allFields) ??
                        CodecArrayFieldGenerator.Create(codecsImporter, fieldInfoAccessor, processedFields, allFields) ??
                        CodecFieldGenerator.Create(codecsImporter, fieldInfoAccessor) ?? // должен быть самым последним
                        throw new Exception($"Unsupported field {field.Name} with type {field.FieldType.FullName} in class {type.Name}.");

                    decodeMethod.Append(AddTabs(generator.GenerateDecode()));
                    encodeMethod.Append(AddTabs(generator.GenerateEncode()));
                    processedFields.Add(field.Name);
                }
                catch (Exception e)
                {
                    throw new Exception($"Generate codec for field {field.Name} in class {type.Name} error: {e.Message}.");
                }
            }

            decodeMethod.AppendLine("}");
            encodeMethod.AppendLine("}");
            result.Append(AddTabs(decodeMethod.ToString()));
            result.Append(AddTabs(encodeMethod.ToString()));

            if (codecsImporter.HasCodecs)
            {
                result.Append(AddTabs(codecsImporter.GenerateVariables()));
                result.Append(AddTabs(codecsImporter.GenerateConstructor()));
            }

            result.Append(AddTabs(GenerateRegistrationMethod()));
            result.AppendLine("}");


            return result.ToString();
        }

        private string GenerateNamespace(string classBody)
        {
            var result = new StringBuilder();
            if (codecNamespace != null && codecNamespace.Trim().Length > 0)
            {
                result.AppendLine("// ReSharper disable once CheckNamespace");
                result.AppendLine($"namespace {codecNamespace}");
                result.AppendLine("{");
                result.Append(AddTabs(classBody));
                result.AppendLine("}");
            }
            else
            {
                result.Append(classBody);
            }


            return result.ToString();
        }

        private string GenerateUsing()
        {
            var result = new StringBuilder();
            result.AppendLine("using Cheetah.Matches.Realtime.Codec;");
            result.AppendLine("using Cheetah.Matches.Realtime.Codec.Formatter;");
            result.AppendLine("using Cheetah.Matches.Realtime.Types;");
            result.AppendLine("using UnityEngine;");
            if (type.Namespace != null)
            {
                result.AppendLine($"using {type.Namespace};");
            }

            result.AppendLine();
            return result.ToString();
        }


        private string GenerateRegistrationMethod()
        {
            var result = new StringBuilder();
            result.AppendLine();
            result.AppendLine("[RuntimeInitializeOnLoadMethod(RuntimeInitializeLoadType.SubsystemRegistration)]");
            result.AppendLine("private static void OnRuntimeMethodLoad()");
            result.AppendLine("{");
            result.AppendLine(codecsImporter.HasCodecs
                ? $"\tCodecRegistryBuilder.RegisterDefault(factory=>new {codecClassName}(factory));"
                : $"\tCodecRegistryBuilder.RegisterDefault(factory=>new {codecClassName}());");

            result.AppendLine("}");
            return result.ToString();
        }
    }

    internal class FieldInfoAccessorImpl : FieldInfoAccessor
    {
        private readonly FieldInfo field;

        public FieldInfoAccessorImpl(FieldInfo field)
        {
            this.field = field;
        }


        public T GetCustomAttribute<T>() where T : Attribute
        {
            return field.GetCustomAttribute<T>();
        }

        public Type FieldType => field.FieldType;
        public string Name => field.Name;
    }

    /// <summary>
    /// Использование внешних кодеков
    /// - создает свойство для хранения кодека
    /// - получает кодек в конструкторе из реестра
    /// </summary>
    public class CodecsImporter
    {
        private readonly string codecClassName;

        private readonly Dictionary<Type, string> types = new();

        public CodecsImporter(string codecClassName)
        {
            this.codecClassName = codecClassName;
        }

        internal string GetCodecName(Type type)
        {
            if (types.TryGetValue(type, out var codec))
            {
                return codec;
            }

            var codecVariableName = "codec" + types.Count;
            types.Add(type, codecVariableName);

            return codecVariableName;
        }

        internal string GenerateVariables()
        {
            var result = new StringBuilder();
            foreach (var (type, codecFieldName) in types)
            {
                result.AppendLine($"private readonly Codec<{GetFullName(type)}> {codecFieldName};");
            }

            return result.ToString();
        }

        internal string GenerateConstructor()
        {
            var result = new StringBuilder();
            result.AppendLine($"private {codecClassName}(CodecRegistry codecRegistry)");
            result.AppendLine("{");
            foreach (var (type, codecFieldName) in types)
            {
                result.AppendLine($"\t{codecFieldName} = codecRegistry.GetCodec<{GetFullName(type)}>();");
            }

            result.AppendLine("}");
            return result.ToString();
        }

        public bool HasCodecs => types.Count > 0;
    }
}