import { Avatar, AvatarFallback } from '@/components/ui/avatar';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { LogoutButton } from '@/components/common/logout-button';
import { User } from '@/types';
import { Skeleton } from '@/components/ui/skeleton';

interface UserProfileDropdownProps {
  user: User | null | undefined;
  isLoading?: boolean;
}

export function UserProfileDropdown({
  user,
  isLoading,
}: UserProfileDropdownProps) {
  const getInitials = (username?: string) => {
    if (!username) return 'U';
    return username.charAt(0).toUpperCase();
  };

  if (isLoading) {
    return <Skeleton className='h-8 w-8 rounded-full' />;
  }

  if (!user) {
    return null;
  }

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <button className='focus:ring-accent-ring rounded-full focus:ring-2 focus:ring-offset-2 focus:outline-none'>
          <Avatar>
            <AvatarFallback>{getInitials(user.username)}</AvatarFallback>
          </Avatar>
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align='end' className='w-56'>
        <DropdownMenuLabel className='truncate font-normal'>
          <div className='flex flex-col space-y-1'>
            <p className='text-sm leading-none font-medium'>Signed in as</p>
            <p className='text-gray-text-muted truncate text-xs leading-none'>
              {user.username}
            </p>
          </div>
        </DropdownMenuLabel>
        <DropdownMenuSeparator />
        <DropdownMenuItem asChild className='cursor-pointer p-0'>
          <LogoutButton className='h-8 w-full justify-start' />
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
