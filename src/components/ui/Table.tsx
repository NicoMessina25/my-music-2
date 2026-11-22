/* import { Table } from '@mantine/core';
import React, { Key } from 'react'

interface ColumnProps<T> {
    header: string
    key: keyof T
}

interface DataTableProps<T> {
    elements: T[],
    columns: ColumnProps<T>[]
    rowKey: keyof T
    className?: string
}

export default function DataTable<T>({elements, columns, rowKey, className=""}:DataTableProps<T>) {
    const rows = elements.map((element) => (
        <Table.Tr key={element[rowKey] as Key}>
            {columns.map((c, i) =>
                <Table.Td key={element[rowKey]+''+i}>{element[c.key]}</Table.Td>
            )}
        </Table.Tr>
    ));

    return (
        <Table className={className}>
            <Table.Thead>
                <Table.Tr>
                    {columns.map((c, i) =>
                        <Table.Th key={i}>{c.header}</Table.Th>
                    )}
                </Table.Tr>
            </Table.Thead>
            <Table.Tbody>{rows}</Table.Tbody>
        </Table>
    );
}
 */