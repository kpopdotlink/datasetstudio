import { Routes, Route } from "react-router-dom";
import { useEffect } from "react";
import Layout from "./components/layout/Layout";
import Home from "./components/project/Home";
import ProjectOverview from "./components/project/ProjectOverview";
import Sources from "./components/sources/Sources";
import ChunkSettings from "./components/chunks/ChunkSettings";
import ReviewQueue from "./components/review/ReviewQueue";
import DatasetManager from "./components/dataset/DatasetManager";
import Exports from "./components/export/Exports";
import { useThemeStore } from "./stores/themeStore";

function App() {
  const { theme } = useThemeStore();

  useEffect(() => {
    const root = window.document.documentElement;
    root.classList.remove("light", "dark");
    root.classList.add(theme);
  }, [theme]);

  return (
    <Routes>
      <Route path="/" element={<Home />} />
      <Route element={<Layout />}>
        <Route path="/project" element={<ProjectOverview />} />
        <Route path="/sources" element={<Sources />} />
        <Route path="/chunks" element={<ChunkSettings />} />
        <Route path="/review" element={<ReviewQueue />} />
        <Route path="/dataset" element={<DatasetManager />} />
        <Route path="/exports" element={<Exports />} />
      </Route>
    </Routes>
  );
}

export default App;
