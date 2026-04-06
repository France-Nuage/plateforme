import {
  Box,
  Center,
  Link as ChakraLink,
  Heading,
  Image,
  Text,
  VStack,
} from '@chakra-ui/react';
import { FunctionComponent } from 'react';
import { Link } from 'react-router';

import { ColorModeButton } from '@/components/chakra';

/**
 * Private beta landing page.
 *
 * Displayed to authenticated users who are not yet members of any organization.
 * Informs them that the console is in private beta and provides a contact email
 * to request early access.
 */
export const PrivateBetaPage: FunctionComponent = () => (
  <Box h="100vh" colorPalette="blue">
    <Box as="header" borderBottomWidth={1} py={2} px={4}>
      <Box display="flex" alignItems="center" gap={4}>
        <Image src="/logo.png" h={10} paddingY={1} alt="France Nuage logo" />
        <Heading size="md">France Nuage</Heading>
        <Box flexGrow={1} />
        <ColorModeButton colorPalette="gray" />
      </Box>
    </Box>
    <Center flex={1} h="calc(100vh - 57px)">
      <VStack gap={6} maxW="lg" textAlign="center" px={4}>
        <Heading size="2xl">Console en bêta privée</Heading>
        <Text fontSize="lg" color="fg.muted">
          La console France Nuage est actuellement en bêta privée. Si vous
          souhaitez bénéficier d'un accès anticipé, contactez-nous à l'adresse
          suivante :
        </Text>
        <ChakraLink
          asChild
          fontSize="lg"
          fontWeight="semibold"
          variant="underline"
        >
          <Link to="mailto:contact@france-nuage.fr">
            contact@france-nuage.fr
          </Link>
        </ChakraLink>
      </VStack>
    </Center>
  </Box>
);
