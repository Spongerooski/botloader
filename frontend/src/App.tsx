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
import { SelectServerPage } from './pages/SelectServer';
import { UserSettingsPage } from './pages/UserSettings';
import { GuildPage } from './pages/GuildPage';

function App() {
  return (
    <Router>
      <Switch>
        <Route path="/confirm_login">
          <ConfirmLoginPage />
        </Route>
        <Route path="/">
          <SessionProvider>
            <Switch>
              <Route path="/settings">
                <TopNav />
                <RequireLoggedInSession>
                  <div className="page-wrapper"><UserSettingsPage /></div>
                </RequireLoggedInSession>
              </Route>
              <Route path="/servers">
                <GuildsProvider>
                  <Switch>
                    <Route path="/servers/:guildId">
                      <RequireLoggedInSession>
                        <OuterGuildPage />
                      </RequireLoggedInSession>
                    </Route>
                    <Route path="/servers">
                      <TopNav />
                      <div className="page-wrapper"><SelectServerPage /></div>
                    </Route>
                  </Switch>
                </GuildsProvider>
              </Route>
              <Route path="/">
                <header className="App-header">
                  <p>BotLoader coming soon™</p>
                  <img src="/logo192.png" alt="zzz"></img>
                </header>
              </Route>
            </Switch>
          </SessionProvider>
        </Route>
      </Switch>
    </Router>
  );
}

function OuterGuildPage() {
  let { guildId }: { guildId: string } = useParams();

  return <CurrentGuildProvider guildId={guildId}>
    <TopNav />
    <div className="page-wrapper">
      <GuildPage />
    </div>
  </CurrentGuildProvider>
}

export default App;
