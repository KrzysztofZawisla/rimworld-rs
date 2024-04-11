using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Reflection.Emit;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;
using System.Text.RegularExpressions;
using System.Xml.Linq;
namespace RustBridge.Smoke;

internal sealed partial class FakeInstruction
{
    public object opcode;

    public object operand;

    public FakeInstruction(object opcode, object operand)
    {
        this.opcode = opcode;
        this.operand = operand;
    }
}
