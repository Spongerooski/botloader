import React from 'react';
import './App.css';
import {
  BrowserRouter as Router,
  Switch,
  Route,
  useParams,
} from "react-router-dom";
import { RequireLoggedInSession, SessionProvider } from './components/Session';
import { CurrentGuildProvider, GuildsProvider } from './components/GuildsProvider';
import { TopNav } from './components/TopNav';
import { ConfirmLoginPage } from './pages/ConfirmLogin';

function App() {
  return (
    <Router>
      <Switch>
        <Route path="/confirm_login">
          <ConfirmLoginPage />
        </Route>

        <Route path="/servers">
          <SessionProvider>
            <GuildsProvider>
              <Switch>
                <Route path="/servers/:guildId">
                  <RequireLoggedInSession>
                    <GuildPage />
                  </RequireLoggedInSession>
                </Route>
                <Route path="/servers">
                  <TopNav />
                  <header><p>List guilds...TODO</p></header>
                </Route>
              </Switch>
            </GuildsProvider>
          </SessionProvider>
        </Route>
        <Route path="/">
          <header className="App-header">
            <p>BotLoader coming soon™</p>
            <img src="/logo192.png" alt="zzz"></img>
          </header>
        </Route>
      </Switch>
    </Router>
  );
}

function GuildPage() {
  let { guildId }: { guildId: string } = useParams();

  return <CurrentGuildProvider guildId={guildId}><TopNav /><p>Current guild page for {guildId}</p></CurrentGuildProvider>

}

export default App;
