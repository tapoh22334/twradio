import React from 'react';
import './App.css';

import { invoke } from '@tauri-apps/api'
import { listen, emit } from '@tauri-apps/api/event'
import { TweetLi, TweetLiProps } from './components/TweetCard'

import List from '@mui/material/List';
import Divider from '@mui/material/Divider';

type ViewElements = {
    tweet_id: string,
    text: string,
    name: string,
    username: string,
}

function App() {

  const [tweetList, setTweetList] = React.useState<Array<TweetLiProps>>(()=>{
    return [ ]
  });

  React.useEffect(() => {
    listen('tauri://frontend/token-register', (event)=> {
        console.log(event);
        localStorage.setItem("token", JSON.stringify(event.payload));
    });

    listen('tauri://frontend/token-request', (event)=> {
        const token = localStorage.getItem("token")
        if (token) {
            const json = JSON.parse(token);
            emit('tauri://backend/token-response', json);

            console.log(json);
        } else {
            emit('tauri://backend/token-response');

            console.log("return none");
        }
    });

    listen<ViewElements>('tauri://frontend/display', (event) => {
        const data: ViewElements = event.payload;
        tweetList.push(
            {tweet_id: data.tweet_id,
            username: data.name,
            user_id: data.username,
            time: "4s",
            tweet: data.text}
        )
        setTweetList([...tweetList]);
    });

    console.log("invoke setup_app function");
    invoke('setup_app').then(() => console.log('setup_app complete'));
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
                                tweet_id={row.tweet_id}
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
