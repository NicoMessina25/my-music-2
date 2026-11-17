import { useState } from 'react'
import { ArtistInput } from '../../api/artists'

export default function ArtistsPage() {
    const [artist, setArtist] = useState<ArtistInput>({ name: "" })

    return <div>
        <input placeholder='Artista' type="text" value={artist.name} onChange={e => setArtist({ name: e.target.value })} />
    </div>
  
}
