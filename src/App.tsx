import React from 'react';
import logo from './logo.svg';
import './App.css';

import TweetLi from './components/TweetCard'

import List from '@mui/material/List';
import Divider from '@mui/material/Divider';

function App() {
  return (
    <div className="App">
        <List
          sx={{
            width: '100%',
            maxWidth: 360,
            bgcolor: 'background.paper',
          }}
        >

          <Divider component="li" />
          <TweetLi />
          <Divider component="li" />
          <TweetLi />
          <Divider component="li" />
          <TweetLi />
          <Divider component="li" />

        </List>
    </div>
  );
}

export default App;
