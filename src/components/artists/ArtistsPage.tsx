import { useEffect, useState } from 'react'
import { ArtistView, listArtists } from '../../api/artists'
import { DataTable } from 'mantine-datatable'
import { showNotification } from '@mantine/notifications';

export default function ArtistsPage() {
    const [artists, setArtists] = useState<ArtistView[]>([])

    useEffect(() => {
        listArtists().then(setArtists).catch(() => {})
    }, [])

    return <div className='pageContainer flex-col'>
        <h1>Artistas</h1>
        <DataTable
            withTableBorder
            borderRadius="sm"
            withColumnBorders
            striped
            highlightOnHover
            records={artists}
            columns={[{
                accessor: 'id',
                title: '#',
                // right-align column
                textAlign: 'right',
            },
            { accessor: 'name' },
            { accessor: 'songsCount' },
            ]}
            // execute this callback when a row is clicked
            onRowClick={({ record: { name, songsCount } }) =>
                showNotification({
                    title: `Clicked on ${name}`,
                    message: `You clicked on ${name}, an artist with ${songsCount} canciones`,
                    withBorder: true,
                })
            }
            emptyState={"No hay artistas"}
        />
        
    </div>
  
}
