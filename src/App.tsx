import React, { useEffect } from "react";
import "./App.css";
import init, { compile, step } from "rispy";
import { Static, Type } from "@sinclair/typebox";
import { Value } from "@sinclair/typebox/value";

const VMInstructionSchema = Type.Union([
  Type.Object({
    Constant: Type.Number(),
  }),
  Type.Object({
    Lookup: Type.String(),
  }),
  Type.Object({
    Call: Type.Number(),
  }),
  Type.String(),
]);
type VMInstruction = Static<typeof VMInstructionSchema>;

const ExprSchema = Type.Union([
  Type.Object({ Num: Type.Number() }),
  Type.Object({ BuiltIn: Type.Array(VMInstructionSchema) }),
]);

const Callframe = Type.Object({
  ip: Type.Number(),
  chunk: Type.Object({
    code: Type.Array(VMInstructionSchema),
    constants: Type.Array(ExprSchema),
  }),
});

const VM = Type.Object({
  callframes: Type.Array(Callframe),
  stack: Type.Array(ExprSchema),
  globals: Type.Record(Type.String(), ExprSchema),
});

type VMType = Static<typeof VM>;

const parseResult = (
  result: unknown,
): { type: "success"; value: VMType } | { type: "error"; error: unknown } => {
  try {
    return { type: "success", value: Value.Decode(VM, result) };
  } catch (error) {
    return { type: "error", error };
  }
};

function App() {
  const [value, setValue] = React.useState("(+ 1 (+ 1 2)) ");
  const [expr, setExpr] = React.useState<unknown>(null);

  useEffect(() => {
    init()
      .then(() => {
        setExpr(compile(value));
      })
      .catch((e) => setExpr(`An error occured: ${e.message}`));
  }, [value]);

  const deserializedResult = parseResult(expr);
  if (deserializedResult.type === "error") {
    console.error(JSON.stringify(deserializedResult.error));
  }

  return (
    <div className="App">
      <header className="App-header">
        <div style={{ display: "flex", flexDirection: "row" }}>
          <div>
            <textarea
              value={value}
              onChange={(e) => {
                setValue(e.target.value);
              }}
            />
            <div
              style={{
                display: "flex",
                flexDirection: "row",
                marginTop: "8px",
                gap: "8px",
              }}
            >
              {deserializedResult.type === "success" ? (
                <button
                  onClick={() => {
                    setExpr(step(deserializedResult.value));
                  }}
                >
                  step
                </button>
              ) : null}
            </div>
          </div>
          <div style={{ marginLeft: "32px" }}>
            {deserializedResult.type === "success" ? (
              <VMComponent vm={deserializedResult.value} />
            ) : (
              "Error, see browser console"
            )}
          </div>
        </div>
      </header>
    </div>
  );
}

const VMInstructionComp = ({
  instr,
  active,
}: {
  instr: VMInstruction;
  active: boolean;
}) => {
  const formatted = React.useMemo(() => {
    if (typeof instr === "string") return instr;

    const entries = Object.entries(instr)[0];

    return `${entries[0]}(${entries[1]})`;
  }, [instr]);

  return (
    <div style={{ backgroundColor: active ? "green" : "grey", padding: "4px" }}>
      {formatted}
    </div>
  );
};

const VMComponent = ({ vm }: { vm: VMType }) => {
  const callframeCount = vm.callframes.length;
  const reversedStack = [...vm.stack].reverse();
  const reversedCallframes = [...vm.callframes].reverse();

  return (
    <div style={{ display: "flex", flexDirection: "row", gap: "16px" }}>
      <div style={{ display: "flex", flexDirection: "column" }}>
        {reversedStack.map((item) => (
          <div>{JSON.stringify(item)}</div>
        ))}
      </div>

      <div style={{ display: "flex", flexDirection: "column", gap: "24px" }}>
        {reversedCallframes.map((callframe, callFrameIndex) => {
          const code = callframe.chunk.code;
          return (
            <div
              style={{
                display: "flex",
                flexDirection: "row",
                gap: "16px",
                opacity: callFrameIndex !== 0 ? "0.5" : "1",
              }}
            >
              {code.map((c, i) => (
                <VMInstructionComp instr={c} active={i === callframe.ip} />
              ))}
            </div>
          );
        })}
      </div>
    </div>
  );
};

export default App;
