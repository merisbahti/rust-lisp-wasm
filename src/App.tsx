import React, { useEffect } from "react";
import logo from "./logo.svg";
import "./App.css";
import * as rsw from "rispy";

function App() {
  const [value, setValue] = React.useState("");
  const [expr, setExpr] = React.useState("");

  useEffect(() => {
    rsw.default().then(() => {
      setExpr(rsw.evaluate_symbolic_string(value));
    });
  }, [rsw, value]);

  return (
    <div className="App">
      <header className="App-header">
        <img src={logo} className="App-logo" alt="logo" />
        <p>Hello WebAssembly!</p>
        <p>Vite + Rust + React</p>
        <input
          type="text"
          value={value}
          onChange={(e) => {
            setValue(e.target.value);
            //setExpr(rsw.greet(e.target.value));
          }}
        />
        <code>{expr}</code>
        <p>
          Edit <code>App.tsx</code> and save to test HMR updates.
        </p>
      </header>
    </div>
  );
}

export default App;
