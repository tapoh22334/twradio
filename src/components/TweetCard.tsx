import './TweetCard.css';

import * as React from 'react';
import ListItem from '@mui/material/ListItem';
import ListItemAvatar from '@mui/material/ListItemAvatar';
import ListItemText from '@mui/material/ListItemText';
import Avatar from '@mui/material/Avatar';
import Typography from '@mui/material/Typography';

export type TweetProps = {
    tweet_id: string,
    username: string,
    user_id: string,
    time: string,
    tweet: string,
    profile_image_url: string,
}

export type TweetLiProps = {
    tweet_id: string,
    username: string,
    user_id: string,
    time: string,
    tweet: string,
    profile_image_url: string,
    focus: boolean,
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

const TweetLiText: React.FC<TweetLiProps> = (props) => {
    return (
        <ListItemText
          primary={
            <React.Fragment>

                {props.username}

                <Typography
                  sx={{ display: 'inline' }}
                  component="span"
                  variant="caption"
                  color="text.primary"
                >

                   {" @" + props.user_id}

                </Typography>
                <Typography
                  sx={{ display: 'inline' }}
                  component="span"
                  variant="caption"
                  color="text.primary"
                >
                   {"・" + format_time(props.time)}
                </Typography>
            </React.Fragment>
          }

          secondary={ props.tweet }

        />
    );
}

export const TweetLi: React.FC<TweetLiProps> = (props) => {
  if (props.focus) {
      return (
          <ListItem id={props.tweet_id} alignItems="flex-start" sx={{margin: "4px", border: "1px solid #6c757d", borderRadius: "8px"}}>
            <ListItemAvatar>
              <Avatar alt="Cindy Baker" src={props.profile_image_url} />
            </ListItemAvatar>
            <TweetLiText {...props} />
          </ListItem>

      );
  } else {
      return (
          <ListItem id={props.tweet_id} alignItems="flex-start" sx={{margin: "4px"}}>
            <ListItemAvatar>
              <Avatar alt="Cindy Baker" src={props.profile_image_url} />
            </ListItemAvatar>
            <TweetLiText {...props} />
          </ListItem>

      );
  }
}
