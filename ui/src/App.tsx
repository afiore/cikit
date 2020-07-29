import React from 'react';
import './App.css';
import * as FailedSuites from './components/FailedSuites';
import * as AllSuites from './components/AllSuites';
import { FailedTestSuite, TestSuite, } from './dtos';

interface AppProps {
  datasetUri: string
}
interface AppState {
  failed: FailedTestSuite[];
  all: TestSuite[];
}

class App extends React.Component<AppProps, AppState> {
  constructor(props: any) {
    super(props);

    this.state = {
      failed: [],
      all: []
    };
  }

  componentDidMount() {
    fetch(this.props.datasetUri)
      .then(response => response.json())
      .then(result => this.setState({
        failed: result.failed,
        all: result.allSuites,
      }));
  }

  render() {
    return (
      <section>
        <FailedSuites.Component failed={this.state.failed} />
        <AllSuites.Component all={this.state.all} />
      </section>
    );
  }
}

export default App;
