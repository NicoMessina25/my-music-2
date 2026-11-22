import { Button } from "@mantine/core";
import { DataTable } from "mantine-datatable";
import { useEffect, useState } from "react";
import { listSongs, SongView } from "../../api/song";
import { showNotification } from "@mantine/notifications";

export default function SongsPage() {
    const [songs, setSongs] = useState<SongView[]>([])

    useEffect(() => {
        listSongs().then(setSongs).catch(err => showNotification({ message: err }))
    }, [])

    return (<div className="pageContainer flex-col">
        <div className='flex justify-between'>
            <h1>Registro de canciones</h1>
            <Button onClick={()=>{}}>Crear</Button>
        </div>
        <DataTable
            withTableBorder
            borderRadius="sm"
            withColumnBorders
            striped
            highlightOnHover
            records={songs}
            columns={[{
                accessor: 'id',
                title: '#',
                textAlign: 'right',
            },
            { accessor: 'title' },
            { accessor: 'artists' },
            ]}
            emptyState={"No hay artistas"}
        />
    </div>)
}
