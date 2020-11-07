import { FunctionalComponent, h } from 'preact';
import { Route, Router } from 'preact-router';
import Home from '../routes/home';

const App: FunctionalComponent<{}> = () => {
  return (
    <div id="app">
      <Router>
        <Route path="/" default component={Home} />
      </Router>
    </div>
  );
};

export default App;
