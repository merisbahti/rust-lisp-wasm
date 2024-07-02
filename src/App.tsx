import React, { useEffect } from "react";
import logo from "./logo.svg";
import "./App.css";
import init, { compile } from "rispy";
import { Static, Type, ValueGuard } from "@sinclair/typebox";
import { Value } from "@sinclair/typebox/value";


const example: VMType = {
  "callframes": [
    {
      "ip": 0, "chunk":
      {
        "code": [
          { "Lookup": "+" },
          { "Constant": 0 },
          { "Constant": 1 },
          { "Call": 2 }, "Return"],
        "constants": [{ "Num": 1 }, { "Num": 2 }]
      }
    }], "stack": [{ "BuiltIn": [] }], "globals": {}
}

const VMInstructionSchema = Type.Union([
  Type.Object({
    Constant: Type.Number()
  }),
  Type.Object({
    Lookup: Type.String()
  }),
  Type.Object({
    Call: Type.Number()
  }),
  Type.Literal("Return")
])
type VMInstruction = Static<typeof VMInstructionSchema>

const ExprSchema = Type.Union([
  Type.Object({ Num: Type.Number() }),
  Type.Object({ BuiltIn: Type.Array(VMInstructionSchema) })
])

const Callframe = Type.Object({
  ip: Type.Number(),
  chunk: Type.Object({
    code: Type.Array(VMInstructionSchema),
    constants: Type.Array(ExprSchema)
  })
})

const VM = Type.Object({
  callframes:
    Type.Array(Callframe),
  stack: Type.Array(ExprSchema),
  globals: Type.Record(Type.String(), ExprSchema)
})

type VMType = Static<typeof VM>

const parseResult = (result: unknown): { type: "success", value: VMType } | { type: "error", error: unknown } => {
  try {
    return { type: "success", value: Value.Decode(VM, result) }
  } catch (error) {
    return { type: "error", error }
  }

}

function App() {
  const [value, setValue] = React.useState("(+ 1 2)");
  const [expr, setExpr] = React.useState<unknown>(null);

  useEffect(() => {
    init()
      .then(() => {
        setExpr(compile(value));
      })
      .catch((e) => setExpr(`An error occured: ${e.message}`));
  }, [init, value]);

  const deserializedResult = parseResult(expr)

  return (
    <div className="App">
      <header className="App-header">
        <img src={logo} className="App-logo" alt="logo" />
        <textarea
          value={value}
          onChange={(e) => {
            setValue(e.target.value);
          }}
        />
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            marginTop: "8px",
            gap: "8px",
          }}
        >
          <div>result:</div>
          {deserializedResult.type === "success" ?
            <VMComponent vm={deserializedResult.value} /> : JSON.stringify(deserializedResult.error)}

        </div>
      </header>
    </div>
  );
}

const VMInstructionComp = ({ instr, active }: { instr: VMInstruction, active: boolean }) => {
  const formatted = React.useMemo(() => {
    if (typeof instr === "string") return instr


    const entries = Object.entries(instr)[0]

    return `${entries[0]}(${entries[1]})`

  }, [instr])

  return <div style={{ backgroundColor: active ? "green" : "grey" }}>{formatted}</div>

}

const VMComponent = ({ vm }: { vm: VMType }) => {
  const callframeCount = vm.callframes.length
  return <div style={{ display: "flex", flexDirection: "row", gap: "16px" }}>
    <div style={{ display: "flex", flexDirection: "row" }}>
      {vm.stack.map(item => <div>{JSON.stringify(item)}</div>)}
    </div>

    <div style={{ display: "flex", flexDirection: "column" }}>
      {vm.callframes.reverse().map((callframe, callFrameIndex) => {
        const code = callframe.chunk.code
        return <div style={{ display: "flex", flexDirection: "row", gap: "16px", opacity: callFrameIndex !== (callframeCount - 1) ? "0.5" : "1" }}>
          {code.map((c, i) => <VMInstructionComp instr={c} active={i === callframe.ip} />)}
        </div>
      })}

    </div>
  </div>
}

export default App;
