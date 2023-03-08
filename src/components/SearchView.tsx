import React from 'react';
import "./SearchView.css";

import { invoke } from "@tauri-apps/api";

import InputBase from '@mui/material/InputBase';
import IconButton from '@mui/material/IconButton';
import SearchIcon from '@mui/icons-material/Search';
import List from '@mui/material/List';
import Divider from '@mui/material/Divider';
import Box from '@mui/material/Box';
import ListItem from '@mui/material/ListItem';
import Paper from '@mui/material/Paper';

import { TweetLi, TweetProps } from './TweetCard';

export const SearchView = ({tweets}: {tweets: Array<TweetProps>}) => {
    const [query, setQuery] = React.useState<string>("")
    const [lastQuery, setLastQuery] = React.useState<string>("")

    const handleSearch = () => {
      if (query != lastQuery && query.length > 1) {
        console.log("handleSearch:" + query)
        setLastQuery(query);
        invoke("set_timeline", {"timeline": {"Search": {"query": query}}} );
      }
    }

    return (
      <React.Fragment>
        <Box
          className="SearchBar"
          display="flex"
          justifyContent="center"
          alignItems="center"
        >
          <Paper
            component="form"
            sx={{ m: '5px 5px', p: '2px 4px', display: 'flex', width: '250px' }}
          >
            <InputBase
              sx={{ ml: 1, flex: 1 }}
              placeholder="Search"
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              inputProps={{ maxLength: 36, 'aria-label': 'search' }}
            />
            <IconButton type="button" sx={{ p: '10px' }} aria-label="search" onClick={handleSearch}>
              <SearchIcon />
            </IconButton>
          </Paper>
        </Box>

        <Box
          className="SearchBody"
        >
          <List
            sx={{
              bgcolor: 'background.paper',
            }}
          >
              {
                  tweets.length > 0 &&
                      tweets.map((row) => {
                          return (
                           <React.Fragment>
                              <TweetLi
                                  tweet_id={row.tweet_id}
                                  author_id={row.author_id}
                                  username={row.username}
                                  user_id={row.user_id}
                                  time={row.time}
                                  tweet={row.tweet}
                                  profile_image_url={row.profile_image_url}
                                  attachments={row.attachments}
                                  focus={false}
                                  />
                              <Divider component="li" />
                           </React.Fragment>
                          )
                      })
              }

              {/* Empty box */}
              <ListItem>
                 <Box
                   sx={{
                     height: "calc(var(--canvas-height) - var(--appbar-height) - var(--footer-height))",
                   }}
                 />
              </ListItem>
          </List>
        </Box>
      </React.Fragment>
    );
}

