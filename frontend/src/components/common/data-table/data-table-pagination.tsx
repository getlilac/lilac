import { Table } from '@tanstack/react-table';

import {
  Pagination,
  PaginationContent,
  PaginationItem,
  PaginationPrevious,
  PaginationEllipsis,
  PaginationLink,
  PaginationNext,
} from '@/components/ui/pagination';

interface DataTablePaginationProps<TData> {
  table: Table<TData>;
}

export function DataTablePagination<TData>({
  table,
}: DataTablePaginationProps<TData>) {
  const pageCount = Math.ceil(
    table.getFilteredRowModel().rows.length /
      table.getState().pagination.pageSize
  );
  const currentIndex = table.getState().pagination.pageIndex;
  const { leftDots, rightDots, start, end } = getPageRange(
    currentIndex,
    pageCount
  );
  const pageOptions = table.getPageOptions().slice(start, end);

  if (pageCount <= 1) {
    return (
      <div className='flex items-center justify-center px-2'>
        <div className='flex items-center space-x-6 lg:space-x-8'>
          <div className='flex items-center space-x-2'>
            <Pagination className='w-full'>
              <PaginationContent className='flex flex-row justify-between'>
                <PaginationItem>
                  <PaginationPrevious onClick={() => table.previousPage()} />
                </PaginationItem>
                <PaginationItem key={1}>
                  <PaginationLink
                    className='transition-none'
                    isActive={0 === currentIndex}
                    onClick={() => table.firstPage()}
                  >
                    {1}
                  </PaginationLink>
                </PaginationItem>
                <PaginationItem>
                  <PaginationNext
                    onClick={() =>
                      currentIndex < pageCount - 1 && table.nextPage()
                    }
                  />
                </PaginationItem>
              </PaginationContent>
            </Pagination>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className='flex items-center justify-center'>
      <div className='flex items-center space-x-6 lg:space-x-8'>
        <div className='flex items-center space-x-2'>
          <Pagination className='w-full'>
            <PaginationContent className='flex flex-row justify-between'>
              <PaginationItem>
                <PaginationPrevious onClick={() => table.previousPage()} />
              </PaginationItem>
              <PaginationItem key={1}>
                <PaginationLink
                  className='transition-none'
                  isActive={0 === currentIndex}
                  onClick={() => table.firstPage()}
                >
                  {1}
                </PaginationLink>
              </PaginationItem>
              {leftDots && <PaginationEllipsis />}
              {pageOptions.map((page) => (
                <PaginationItem key={page}>
                  <PaginationLink
                    className='transition-none'
                    isActive={page === currentIndex}
                    onClick={() => table.setPageIndex(page)}
                  >
                    {page + 1}
                  </PaginationLink>
                </PaginationItem>
              ))}
              {rightDots && <PaginationEllipsis />}
              <PaginationItem>
                <PaginationLink
                  className='transition-none'
                  isActive={pageCount - 1 === currentIndex}
                  onClick={() => table.lastPage()}
                >
                  {pageCount}
                </PaginationLink>
              </PaginationItem>
              <PaginationItem>
                <PaginationNext
                  onClick={() =>
                    currentIndex < pageCount - 1 && table.nextPage()
                  }
                />
              </PaginationItem>
            </PaginationContent>
          </Pagination>
        </div>
      </div>
    </div>
  );
}

function getPageRange(currentIndex: number, pageCount: number) {
  const pagesToShow = 7;
  const leftDelta = Math.floor(pagesToShow / 2);
  const rightDelta = leftDelta;

  const lowerLimit = 1;
  const upperLimit = pageCount - 1;

  let start = currentIndex - leftDelta;
  let end = currentIndex + rightDelta;

  if (start < lowerLimit) {
    end += lowerLimit - start;
    start = lowerLimit;
  }

  if (end > upperLimit) {
    start -= end - upperLimit;
    end = upperLimit;
  }

  start = Math.max(start, lowerLimit);
  end = Math.min(end, upperLimit);

  const leftDots = start > lowerLimit;
  const rightDots = end < upperLimit;

  if (leftDots) {
    start++;
  }

  if (rightDots) {
    end--;
  }

  return { leftDots, rightDots, start, end };
}
