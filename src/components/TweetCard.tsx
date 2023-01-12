import './TweetCard.css';

import * as React from 'react';
import ListItem from '@mui/material/ListItem';
import ListItemAvatar from '@mui/material/ListItemAvatar';
import ListItemText from '@mui/material/ListItemText';
import Avatar from '@mui/material/Avatar';
import Typography from '@mui/material/Typography';

export type TweetLiProps = {
    tweet_id: String,
    username: String,
    user_id: String,
    time: String,
    tweet: String
}

export const TweetLi: React.FC<TweetLiProps> = (props) => {
  return (
      <ListItem alignItems="flex-start">
        <ListItemAvatar>
          <Avatar alt="Cindy Baker" src="/static/images/avatar/3.jpg" />
        </ListItemAvatar>
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
                   {" — " + props.time}
                </Typography>
            </React.Fragment>
          }

          secondary={ props.tweet }

        />
      </ListItem>

  );
}
