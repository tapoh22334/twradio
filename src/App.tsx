import React from 'react';
import './App.css';

import { invoke } from '@tauri-apps/api'
import { listen } from '@tauri-apps/api/event'
import { TweetLi, TweetLiProps } from './components/TweetCard'

import List from '@mui/material/List';
import Divider from '@mui/material/Divider';

function App() {

  const [tweetList, setTweetList] = React.useState<Array<TweetLiProps>>(()=>{
    return [
    {username:"UserName", user_id:"user_id", time:"4s", tweet:"hello world"},
    {username:"UserName", user_id:"user_id", time:"4s", tweet:"hello world"},
    {username:"UserName", user_id:"user_id", time:"4s", tweet:"hello world"},
    {username:"UserName", user_id:"user_id", time:"4s", tweet:"hello world"},
    {username:"UserName", user_id:"user_id", time:"4s", tweet:"hello world"},
    {username:"UserName", user_id:"user_id", time:"4s", tweet:"hello world"},
    {username:"UserName", user_id:"user_id", time:"4s", tweet:"hello world"},
    {username:"UserName", user_id:"user_id", time:"4s", tweet:"hello world"},
    {username:"UserName", user_id:"user_id", time:"4s", tweet:"hello world"},
    {username:"UserName", user_id:"user_id", time:"4s", tweet:"hello world"},
    {username:"UserName", user_id:"user_id", time:"4s", tweet:"hello world"},
    {username:"UserName", user_id:"user_id", time:"4s", tweet:"hello world"},
    {username:"UserName", user_id:"user_id", time:"4s", tweet:"hello world"}];
  });

  React.useEffect(() => {
    const _unlisten = listen('tweet', (event)=> {
        console.log(event);
    });
  }, []) ;

  return (
    <div className="App">
        <List
          sx={{
            width: '100%',
            maxWidth: 360,
            bgcolor: 'background.paper',
          }}
        >
            {
                tweetList.length > 0 &&
                    tweetList.map((row) => {
                        return (
                         <React.Fragment>
                            <TweetLi
                                username={row.username}
                                user_id={row.user_id}
                                time={row.time}
                                tweet={row.tweet} />
                            <Divider component="li" />
                         </React.Fragment>
                        )
                    })
            }
        </List>
    </div>
  );
}

export default App;
