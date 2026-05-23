import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import "./App.css";

interface Game {
  name: string;
  path: string;
  platform: string;
  icon?: string;
}

export default function App() {
  const [games, setGames] = useState<Game[]>([]);

  useEffect(() => {
    invoke<Game[]>("get_installed_games").then(setGames);
  }, []);

  return (
    <div className="app">
      <h1>LaunchPad Library</h1>
      <div className="game-library">
        {games.map((game) => (
          <div key={game.path} className="game-card">
            {game.icon ? (
              <img src={`file://${game.icon}`} alt={game.name} />
            ) : (
              <div className="placeholder-icon" />
            )}
            <h3>{game.name}</h3>
            <p>{game.platform}</p>
          </div>
        ))}
      </div>
    </div>
  );
}
