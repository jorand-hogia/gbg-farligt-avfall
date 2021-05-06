import { createContext, FunctionalComponent, h } from 'preact';
import { useEffect, useState } from 'preact/hooks';
import { Route, Router } from 'preact-router';
import { ApiClient } from '../api/apiClient';
import { Stop as StopType } from '../types/Stop';
import List from '../routes/list';
import Details from '../routes/details';
import VerifySubscription from '../routes/verify';

export const StopsContext = createContext<StopType[]>([]);

const App: FunctionalComponent<{}> = () => {
  const [stops, setStops] = useState<StopType[]>([]);
  useEffect(() => {
    const apiClient = new ApiClient(API_URL);
    apiClient
      .getStops()
      .then(stops => {
        setStops(stops);
      })
      .catch(err => {
        console.error(err);
      });
  }, []);

  return (
    <div id="app">
      <StopsContext.Provider value={stops}>
        <Router>
          <Route path="/" component={List} default />
          <Route path="/details/:locationId" component={Details} />
          <Route path="/verify" component={VerifySubscription} />
        </Router>
      </StopsContext.Provider>
    </div>
  );
};

export default App;
