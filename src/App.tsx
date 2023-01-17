import React from 'react';
import './App.css';

import { invoke } from '@tauri-apps/api'
import { listen, emit } from '@tauri-apps/api/event'
import { TweetLi, TweetLiProps } from './components/TweetCard'

import List from '@mui/material/List';
import Divider from '@mui/material/Divider';
import AppBar from '@mui/material/AppBar';
import Toolbar from '@mui/material/Toolbar';
import Slider from '@mui/material/Slider';
import VolumeUp from '@mui/icons-material/VolumeUp';
import IconButton from '@mui/material/IconButton';
import MenuIcon from '@mui/icons-material/Menu';
import PauseRounded from '@mui/icons-material/PauseRounded';
import PlayArrowRounded from '@mui/icons-material/PlayArrowRounded';
import FastForwardRounded from '@mui/icons-material/FastForwardRounded';
import AdjustIcon from '@mui/icons-material/Adjust';

type ViewElements = {
    tweet_id: string,
    created_at: string,
    text: string,
    name: string,
    username: string,
}

const format_time = (utc: string) => {
    const twtime = new Date(utc).getTime();
    const now = new Date().getTime();

    //1000(ミリ秒) × 60(秒) × 60(分) × 24(時間) = 86400000
    const year = 86400000 * 365;
    const week = 86400000 * 7;
    const day =  86400000;
    const hour =  3600000;
    const min =     60000;
    const sec =      1000;

    const sub = now - twtime;
    let res;
    if (sub > year) {
        res = Math.floor(sub / year).toString() + "y";
    } else if (sub > week) {
        res = Math.floor(sub / week).toString() + "w";
    } else if (sub > day) {
        res = Math.floor(sub / day).toString() + "d";
    } else if (sub > hour) {
        res = Math.floor(sub / hour).toString() + "h";
    } else if (sub > min) {
        res = Math.floor(sub / min).toString() + "m";
    } else if (sub > sec) {
        res = Math.floor(sub / sec).toString() + "s";
    } else {
        res = "1s";
    }
    return res;
}

function App() {

  const [tweetList, setTweetList] = React.useState<Array<TweetLiProps>>(()=>{
    return [ ]
  });

  const [paused, setPaused] = React.useState(false);
  const onPlayStopClick = () => {
    emit('tauri://backend/playstop', !paused);
    setPaused(!paused);
  }


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
            time: data.created_at,
            tweet: data.text}
        )
        setTweetList([...tweetList]);
    });

    listen<string>('tauri://frontend/scroll', (event) => {
        const twid: string = event.payload;
        const targetEl = document.getElementById(twid)
        targetEl?.scrollIntoView({ behavior: 'smooth' })

        console.log(twid);
    });

    console.log("invoke setup_app function");
    invoke('setup_app').then(() => console.log('setup_app complete'));
  }, []) ;

  return (
    <div className="App">
        <AppBar className="Head" position="sticky">
            <Toolbar>
                <IconButton
                  size="large"
                  edge="start"
                  color="inherit"
                  aria-label="menu"
                  sx={{ mr: 0 }}
                >
                    <MenuIcon />
                </IconButton>

                <IconButton
                    color="inherit"
                    onClick={onPlayStopClick}
                >
                    {paused ? (
                        <PlayArrowRounded />
                        ) : (
                        <PauseRounded />
                    )}
                </IconButton>
                <IconButton
                    color="inherit">
                    <FastForwardRounded />
                </IconButton>

                <IconButton
                    color="inherit">
                    <AdjustIcon />
                </IconButton>

                <VolumeUp 
                  sx={{ mr: 1 }}
                />
                <Slider sx={{ width: '40%', color: "inherit"}}/>

            </Toolbar>
        </AppBar>

        <div className="Body">
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
                                    time={format_time(row.time)}
                                    tweet={row.tweet} />
                                <Divider component="li" />
                             </React.Fragment>
                            )
                        })
                }
            </List>
        </div>
    </div>
  );
}

export default App;
