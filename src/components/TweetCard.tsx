import './TweetCard.css';

import * as React from 'react';
import List from '@mui/material/List';
import ListItem from '@mui/material/ListItem';
import ListItemAvatar from '@mui/material/ListItemAvatar';
import ListItemText from '@mui/material/ListItemText';
import Avatar from '@mui/material/Avatar';
import BeachAccessIcon from '@mui/icons-material/BeachAccess';
import Divider from '@mui/material/Divider';
import Typography from '@mui/material/Typography';

export default function TweetLi() {
  return (
      <ListItem alignItems="flex-start">
        <ListItemAvatar>
          <Avatar alt="Cindy Baker" src="/static/images/avatar/3.jpg" />
        </ListItemAvatar>
        <ListItemText
          primary={
            <React.Fragment>
                {'Oui Oui'}
                <Typography
                  sx={{ display: 'inline' }}
                  component="span"
                  variant="caption"
                  color="text.primary"
                >
                   {" @oui_oui"}
                </Typography>
                <Typography
                  sx={{ display: 'inline' }}
                  component="span"
                  variant="caption"
                  color="text.primary"
                >
                   {" — July 20, 2014"}
                </Typography>
            </React.Fragment>
          }
          secondary={
              'Do you have Paris recommendations? Have you ever…'
          }
        />
      </ListItem>

  );
}
