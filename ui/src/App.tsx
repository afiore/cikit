import React from 'react';
import './App.css';
import * as FailedSuites from './components/FailedSuites';
import * as AllSuites from './components/AllSuites';
import { FailedTestSuite, Summary, TestSuite, } from './dtos';
import { SummaryFragment } from './components/Summary';

interface AppProps {
  datasetUri: string
}
interface AppState {
  failed: FailedTestSuite[];
  all: TestSuite[];
  summary: Summary;
}

class App extends React.Component<AppProps, AppState> {
  constructor(props: any) {
    super(props);

    this.state = {
      summary: {
        time: 0,
        tests: 0,
        failures: 0,
        errors: 0,
        skipped: 0,
      },
      failed: [],
      all: []
    };
  }

  componentDidMount() {
    fetch(this.props.datasetUri)
      .then(response => response.json())
      .then(result => this.setState({
        summary: result.summary,
        failed: result.failed,
        all: result.allSuites,
      }));
  }

  render() {
    return (
      <section>
        <SummaryFragment summary={this.state.summary} />
        {(this.state.failed.length > 0) ? <FailedSuites.Component failed={this.state.failed} /> : (<></>)}
        <AllSuites.Component all={this.state.all} />
      </section>
    );
  }
}

export default App;
