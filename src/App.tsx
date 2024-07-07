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
  Type.Object({
    Define: Type.String(),
  }),
  Type.String(),
]);
type VMInstruction = Static<typeof VMInstructionSchema>;

const ExprSchema = Type.Recursive((This) =>
  Type.Union([
    Type.String(),
    Type.Object({ Num: Type.Number() }),
    Type.Object({ BuiltIn: Type.Array(VMInstructionSchema) }),
    Type.Object({
      Lambda: Type.Tuple([
        Type.Object({
          code: Type.Array(VMInstructionSchema),
          constants: Type.Array(This),
        }),
        Type.Array(Type.String()),
        Type.String(),
      ]),
    }),

    Type.Object({
      LambdaDefinition: Type.Tuple([
        Type.Object({
          code: Type.Array(VMInstructionSchema),
          constants: Type.Array(This),
        }),
        Type.Array(Type.String()),
      ]),
    }),
  ]),
);

const ChunkSchema = Type.Recursive((_) =>
  Type.Object({
    code: Type.Array(VMInstructionSchema),
    constants: Type.Array(ExprSchema),
  }),
);
const Callframe = Type.Object({
  ip: Type.Number(),
  chunk: ChunkSchema,
});

const VM = Type.Object({
  callframes: Type.Array(Callframe),
  stack: Type.Array(ExprSchema),
  globals: Type.Record(Type.String(), ExprSchema),
});

type VMType = Static<typeof VM>;

const ResultSchema = Type.Union([
  Type.Object({ Ok: VM }),
  Type.Object({ Err: Type.String() }),
]);

const parseResult = (result: unknown): { Ok: VMType } | { Err: string } => {
  try {
    return Value.Decode(ResultSchema, result);
  } catch (error) {
    console.log(result);
    console.error("failed when serializing:", [
      ...Value.Errors(ResultSchema, result),
    ]);
    return { Err: "Decoding error, look at the console" };
  }
};

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

const StackComp = ({ stackItem }: { stackItem: VMType["stack"][number] }) => {
  const formatted = React.useMemo(() => {
    if (typeof stackItem === "string") {
      return stackItem;
    }
    if ("Num" in stackItem) {
      return stackItem.Num;
    } else if ("BuiltIn") {
      return "fn";
    }
    return "unknown";
  }, [stackItem]);
  return (
    <div style={{ backgroundColor: "grey", padding: "4px", minWidth: "200px" }}>
      {formatted}
    </div>
  );
};

const VMComponent = ({ vm }: { vm: VMType }) => {
  const reversedStack = [...vm.stack].reverse();
  const reversedCallframes = [...vm.callframes].reverse();

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "row",
        gap: "16px",
        height: "80vh",
      }}
    >
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          gap: "8px",
        }}
      >
        {reversedStack.map((item, index) => (
          <div key={index} style={{ padding: "8px", backgroundColor: "grey" }}>
            <StackComp stackItem={item} />
          </div>
        ))}
      </div>

      <div style={{ display: "flex", flexDirection: "column", gap: "24px" }}>
        {reversedCallframes.map((callframe, callFrameIndex) => {
          const code = callframe.chunk.code;
          return (
            <div
              key={callFrameIndex}
              style={{
                display: "flex",
                flexDirection: "row",
                flexWrap: "wrap",
                gap: "16px",
                opacity: callFrameIndex !== 0 ? "0.5" : "1",
              }}
            >
              {code.map((c, i) => (
                <VMInstructionComp
                  key={i}
                  instr={c}
                  active={i === callframe.ip}
                />
              ))}
            </div>
          );
        })}
      </div>
    </div>
  );
};

function App() {
  const [value, setValue] = React.useState(
    `
(define x 1)
(define fnA (lambda () 
    (+ x y)
))
(define fnB (lambda () 
    (define y 5)
    (fnA)
))
(fnB)
`.trim(),
  );
  const [expr, setExpr] = React.useState<{
    previousResult: unknown;
    result: unknown;
  }>({ previousResult: null, result: null });

  useEffect(() => {
    init()
      .then(() => {
        try {
          setExpr(({ previousResult }) => ({
            previousResult,
            result: compile(value),
          }));
        } catch (e) {
          console.log("error");
        }
      })
      .catch((e) =>
        setExpr(() => ({
          previousResult: null,
          result: `An error occured: ${e.message}`,
        })),
      );
  }, [value]);

  const previousResultDeserialized =
    expr.previousResult !== null ? parseResult(expr.previousResult) : null;
  const deserializedResult =
    expr.result !== null ? parseResult(expr.result) : null;
  console.log(JSON.stringify(deserializedResult, null, 2));

  return (
    <div className="App">
      <header className="App-header">
        <div style={{ display: "flex", flexDirection: "row" }}>
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              alignItems: "end",
            }}
          >
            <textarea
              rows={10}
              style={{ fontSize: "32px", width: "100%" }}
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
              {deserializedResult && "Ok" in deserializedResult ? (
                <button
                  onClick={() => {
                    setExpr(() => {
                      return {
                        previousResult: deserializedResult,
                        result: step(deserializedResult.Ok),
                      };
                    });
                  }}
                >
                  step
                </button>
              ) : null}
            </div>
          </div>
          <div style={{ marginLeft: "32px" }}>
            {deserializedResult && "Ok" in deserializedResult ? (
              <VMComponent vm={deserializedResult.Ok} />
            ) : (
              <div style={{ display: "flex", flexDirection: "column" }}>
                <div style={{ color: "red" }}>{deserializedResult?.Err}</div>
                {previousResultDeserialized &&
                "Ok" in previousResultDeserialized ? (
                  <VMComponent vm={previousResultDeserialized.Ok} />
                ) : null}
              </div>
            )}
          </div>
        </div>
      </header>
    </div>
  );
}

export default App;
