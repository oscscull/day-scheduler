import { mount } from "svelte";

import Root from "./Root.svelte";
import "./styles/tokens.css";
import "./styles/base.css";

const target = document.getElementById("app");
if (target) mount(Root, { target });
