import React, { useEffect } from "react";
import logo from "./logo.svg";
import "./App.css";
import init, { evaluate_symbolic_string } from "rispy";

function App() {
  const [value, setValue] = React.useState("");
  const [expr, setExpr] = React.useState("");

  useEffect(() => {
    init().then(() => {
      setExpr(evaluate_symbolic_string(value));
    });
  }, [init, value]);

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
          <code>{expr}</code>
        </div>
      </header>
    </div>
  );
}

export default App;
