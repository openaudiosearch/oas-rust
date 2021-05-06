import React, { useState, useContext, useMemo } from 'react'
import { Flex, Stack, Box, Text, Heading, IconButton, Input, Button, useDisclosure } from '@chakra-ui/react'
import {
  FaPlay as PlayIcon,
  FaHeart as HeartIcon,
  FaSkull as HateIcon
} from 'react-icons/fa'

const PlayerContext = React.createContext(null)

export function PlayerProvider (props) {
  const { children } = props

  const [track, setTrack] = useState(null)

  const context = useMemo(() => ({
    track,
    setTrack
  }), [track])

  return (
    <PlayerContext.Provider value={context}>
      {children}
    </PlayerContext.Provider>
  )
}

export function usePlayer () {
  const context = useContext(PlayerContext)
  return context
}

export function Player (props = {}) {
  const { track } = usePlayer()
  console.log('player', track)
  return (
    <Box>
      {track && (
        // Remove html highlighting tags from title display in player
        <Box>Currently playing: {track.headline.replace(/(<([^>]+)>)/gi, "")};
          <audio controls="controls" key={track.headline}>
          <source src={track.contentUrl} type="audio/mpeg" />
          Your browser does not support the audio element.
          </audio>
        </Box>
      )}
      {!track && (
        <Box>Not playing anything</Box>
      )}
    </Box>
  )
}
