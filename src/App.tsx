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
import Alert from '@mui/material/Alert';
import Drawer from '@mui/material/Drawer';
import Box from '@mui/material/Box';
import ListItem from '@mui/material/ListItem';
import ListItemButton from '@mui/material/ListItemButton';
import ListItemIcon from '@mui/material/ListItemIcon';
import ListItemText from '@mui/material/ListItemText';
import MailIcon from '@mui/icons-material/Mail';
import SettingsIcon from '@mui/icons-material/Settings';
import AbcIcon from '@mui/icons-material/Abc';

type ViewElements = {
    tweet_id: string,
    created_at: string,
    text: string,
    name: string,
    username: string,
    profile_image_url: string,
}

function App() {

  const [focusTwid, setFocusTwid] = React.useState<string>(()=>{
    return ""
  });

  const [tweetList, setTweetList] = React.useState<Array<TweetLiProps>>(()=>{
    return [ ]
  });

  const [volume, setVolume] = React.useState(() => {
    const json = localStorage.getItem("volume");
    const initVolume = json === null ? null : JSON.parse(json);

    return initVolume === null ? 80 : initVolume;
  });

  const onVolumeChange = (event: Event, newValue: number | number[]) => {
    setVolume(newValue as number);
    invoke('set_volume', {volume: newValue as number});
    localStorage.setItem("volume", JSON.stringify(newValue as number));
  };

  const [paused, setPaused] = React.useState(true);
  const onPauseResumeClick = () => {
    setPaused(!paused);
    invoke('set_paused', {paused: !paused});
  }

  const onFocusClick = () => {
      const targetEl = document.getElementById(focusTwid)
      targetEl?.scrollIntoView({ behavior: 'smooth' })
  }

  const onSkipClick = () => {
    const index = tweetList.findIndex((elem) => elem.tweet_id === focusTwid);
    let id;
    if (index in tweetList) {
        id = tweetList[index + 1]?.tweet_id;
    } else {
        id = "";
    }

    invoke('jump', {twid: id});
  }

  const [drawerState, setDrawerState] = React.useState(true);

  const toggleDrawer =
    (open: boolean) =>
    (event: React.KeyboardEvent | React.MouseEvent) => {
      if (
        event.type === 'keydown' &&
        ((event as React.KeyboardEvent).key === 'Tab' ||
          (event as React.KeyboardEvent).key === 'Shift')
      ) {
        return;
      }

      setDrawerState(open);
    };

  const drawerElements = () => (
    <Box
      sx={{ width: `var(--drawer-width)` }}
      role="presentation"
      onClick={toggleDrawer(false)}
      onKeyDown={toggleDrawer(false)}
    >
      <List>

      <Divider />

      <ListItem key='Timeline' disablePadding>
        <ListItemButton>
          <ListItemIcon>
            <AbcIcon />
          </ListItemIcon>
          <ListItemText primary='Timeline' />
        </ListItemButton>
      </ListItem>

      <Divider />

      <ListItem key='Settings' disablePadding>
        <ListItemButton>
          <ListItemIcon>
            <SettingsIcon />
          </ListItemIcon>
          <ListItemText primary='Settings' />
        </ListItemButton>
      </ListItem>

      <Divider />

      </List>
    </Box>
  );

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

    listen<ViewElements>('tauri://frontend/display/add', (event) => {
        const data: ViewElements = event.payload;
        tweetList.push(
            {tweet_id: data.tweet_id,
            username: data.name,
            user_id: data.username,
            time: data.created_at,
            tweet: data.text,
            profile_image_url: data.profile_image_url
            }
        )
        setTweetList([...tweetList]);
    });

    listen<string>('tauri://frontend/display/delete', (event) => {
        const twid: string = event.payload;
        const index = tweetList.findIndex((elem) => elem.tweet_id === twid);
        tweetList.splice(index, 1);
        setTweetList([...tweetList]);
    });

    listen<string>('tauri://frontend/display/scroll', (event) => {
        const twid: string = event.payload;
        const targetEl = document.getElementById(twid)
        targetEl?.scrollIntoView({ behavior: 'smooth' })
        setFocusTwid(twid);
        console.log(twid);
    });

    console.log("invoke setup_app function");
    invoke('setup_app').then(() => console.log('setup_app complete'));
  }, []) ;


  React.useEffect(() => {
        const targetEl = document.getElementById(focusTwid)

        if (targetEl) {
            targetEl?.scrollIntoView({ behavior: 'smooth' })
            console.log(focusTwid);
        }

  }, [focusTwid]);

  return (
    <Box className="App" >
        <Toolbar/>

        <Box sx={{ display: 'flex' }}>
            <AppBar className="Head" position="fixed"
                sx={{
                  width: `calc(100% - var(--drawer-width))`,
                  ml: `var(--drawer-width)`,
                }}>

                <Toolbar>
                    {/*
                    <IconButton
                      size="large"
                      edge="start"
                      color="inherit"
                      aria-label="menu"
                      sx={{ mr: 0 }}
                    >
                        <MenuIcon onClick={toggleDrawer(true)}/>
                    </IconButton>
                    */}

                    <IconButton
                        color="inherit"
                        onClick={onPauseResumeClick}
                    >
                        {paused ? (
                            <PlayArrowRounded />
                            ) : (
                            <PauseRounded />
                        )}
                    </IconButton>

                    <IconButton
                        color="inherit"
                        onClick={onSkipClick}>
                        <FastForwardRounded />
                    </IconButton>

                    <IconButton
                        color="inherit"
                        onClick={onFocusClick}>
                        <AdjustIcon />
                    </IconButton>

                    <VolumeUp 
                      sx={{ mr: 1 }}
                    />
                    <Slider value={volume}
                        onChange={onVolumeChange}
                        min={0}
                        max={100}
                        sx={{ width: '40%', color: "inherit"}}/>

                </Toolbar>
            </AppBar>

            <Box className="SideBar" >
                {drawerElements()}
            </Box>

            <Box className="Body" >
                <List
                  sx={{
                    //maxWidth: 360,
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
                                        tweet={row.tweet}
                                        profile_image_url={row.profile_image_url} />
                                    <Divider component="li" />
                                 </React.Fragment>
                                )
                            })
                    }
                </List>
            </Box>

        </Box>

        <Alert className="Foot" severity="info">バグ報告等 Twitter @tapoh22334</Alert>
    </Box>
  );
}

export default App;
