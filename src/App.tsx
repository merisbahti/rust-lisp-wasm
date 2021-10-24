import React, { useEffect } from "react";
import logo from "./logo.svg";
import "./App.css";
import init, { greet } from "@rsw/hey";

function App() {
  useEffect(() => {
    init();
  }, []);

  const [value, setValue] = React.useState("");
  const [expr, setExpr] = React.useState("");
  React.useEffect(() => {
    /*
     */
  }, [value]);

  return (
    <div className="App">
      <header className="App-header">
        <img src={logo} className="App-logo" alt="logo" />
        <p>Hello WebAssembly!</p>
        <p>Vite + Rust + React</p>
        <input
          type="text"
          value={value}
          onChange={(e) => setValue(greet(e.target.value))}
        />
        <p>
          Edit <code>App.tsx</code> and save to test HMR updates.
        </p>
      </header>
    </div>
  );
}

export default App;
