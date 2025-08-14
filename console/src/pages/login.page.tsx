import {
  Box,
  Button,
  Center,
  Link as ChakraLink,
  Container,
  Flex,
  HStack,
  Heading,
  Image,
  Span,
  Stack,
  Text,
} from '@chakra-ui/react';
import { FunctionComponent } from 'react';
import { BsChevronRight, BsGitlab, BsImage } from 'react-icons/bs';
import { Link } from 'react-router';

import { userManager } from '@/services';

export const LoginPage: FunctionComponent = () => (
  <Flex h="100vh" flex="1" colorPalette="blue">
    <Flex direction="column" flex="1.5">
      <HStack justify="space-between" px={{ base: 4, md: 8 }} minH={16}>
        <Flex alignItems="center" gap={4}>
          <Image src="/logo.png" py={4} w={24} />
          <Heading>France Nuage</Heading>
        </Flex>
        <ChakraLink asChild>
          <Link to="mailto:support@france-nuage.fr">
            Un problème? Contactez-nous
          </Link>
        </ChakraLink>
      </HStack>

      <Container h="full" maxW="md" flex="1" py={{ base: 24, md: 32 }}>
        <Stack justifyContent="center" h="full" gap={8}>
          <Stack alignItems="center" gap={{ base: 2, md: 3 }}>
            <Heading size={{ base: '2xl', md: '3xl' }}>Se connecter</Heading>
            <Text color="fg.muted">
              Nouveau sur notre plateforme?{' '}
              <ChakraLink asChild>
                <Link to="mailto:support@france-nuage.fr">
                  Demander l'accès
                </Link>
              </ChakraLink>
            </Text>
          </Stack>
          <Stack gap={4}>
            <Button
              colorPalette="gray"
              onClick={() => userManager.signinRedirect()}
              rounded="full"
              variant="outline"
              _icon={{ boxSize: '1em' }}
            >
              <BsGitlab />
              <Span flex={1}>Se connecter avec Gitlab</Span>
              <BsChevronRight />
            </Button>
          </Stack>
        </Stack>
      </Container>
    </Flex>
    <Box flex="1" hideBelow="lg">
      <Center
        w="full"
        h="full"
        bg="colorPalette.solid"
        color="colorPalette.contrast/60"
      >
        <BsImage size="40px" />
      </Center>
    </Box>
  </Flex>
);
